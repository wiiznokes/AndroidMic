use std::sync::{LazyLock, Mutex};

use rubato::Resampler;

struct ResamplerCache {
    input_rate: u32,
    output_rate: u32,
    num_channels: usize,
    unprocessed_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
    resampler: rubato::FastFixedIn<f32>,
}

static RESAMPLER_CACHE: LazyLock<Mutex<Option<ResamplerCache>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn resample_f32_stream(
    data: &[Vec<f32>],
    input_sample_rate: u32,
    output_sample_rate: u32,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let chunk_size = 1024;
    let resample_ratio = output_sample_rate as f64 / input_sample_rate as f64;
    let mut resampler_cache = RESAMPLER_CACHE.lock().unwrap();

    if match resampler_cache.as_ref() {
        Some(c) => {
            c.input_rate != input_sample_rate
                || c.output_rate != output_sample_rate
                || c.num_channels != data.len()
        }
        None => true,
    } {
        let resampler = rubato::FastFixedIn::<f32>::new(
            resample_ratio,
            1.0,
            rubato::PolynomialDegree::Cubic,
            chunk_size,
            data.len(),
        )?;

        *resampler_cache = Some(ResamplerCache {
            input_rate: input_sample_rate,
            output_rate: output_sample_rate,
            num_channels: data.len(),
            unprocessed_buffer: vec![Vec::new(); data.len()],
            output_buffer: resampler.output_buffer_allocate(true),
            resampler,
        });
    }

    let cache = resampler_cache.as_mut().unwrap();

    let mut output: Vec<Vec<f32>> =
        vec![
            Vec::with_capacity(cache.output_buffer[0].len() * data[0].len() / chunk_size);
            data.len()
        ];

    let start = if !cache.unprocessed_buffer[0].is_empty() {
        let missing = chunk_size - cache.unprocessed_buffer[0].len();
        if data[0].len() >= missing {
            for (data, unprocessed_buffer) in data.iter().zip(cache.unprocessed_buffer.iter_mut()) {
                unprocessed_buffer.extend_from_slice(&data[0..missing]);
            }

            let (_, frames) = cache.resampler.process_into_buffer(
                &cache.unprocessed_buffer,
                &mut cache.output_buffer,
                None,
            )?;

            for (output, chunk) in output.iter_mut().zip(cache.output_buffer.iter()) {
                output.extend_from_slice(&chunk[..frames]);
            }

            for channel in &mut cache.unprocessed_buffer {
                channel.clear();
            }
            missing
        } else {
            0
        }
    } else {
        0
    };

    let mut data = data
        .iter()
        .map(|channel_data| channel_data[start..].chunks_exact(chunk_size))
        .collect::<Vec<_>>();

    let mut buffer = Vec::with_capacity(data.len());

    let mut done = false;
    loop {
        for chunk in &mut data {
            if let Some(chunk) = chunk.next() {
                buffer.push(chunk);
            } else {
                done = true;
            }
        }

        if done {
            for (chunk, unprocessed_buffer) in data.iter().zip(cache.unprocessed_buffer.iter_mut())
            {
                unprocessed_buffer.extend_from_slice(chunk.remainder());
            }
            return Ok(output);
        }

        let (_, frames) =
            cache
                .resampler
                .process_into_buffer(&buffer, &mut cache.output_buffer, None)?;

        for (output, chunk) in output.iter_mut().zip(cache.output_buffer.iter()) {
            output.extend_from_slice(&chunk[..frames]);
        }

        buffer.clear();
    }
}
