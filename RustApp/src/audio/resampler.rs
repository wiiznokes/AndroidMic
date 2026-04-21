use std::sync::{LazyLock, Mutex};

use rubato::Resampler;

struct ResamplerCache {
    input_rate: u32,
    output_rate: u32,
    num_channels: usize,
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
            resampler,
        });
    }

    let cache = resampler_cache.as_mut().unwrap();

    let mut chunk_output: Vec<Vec<f32>> = cache.resampler.output_buffer_allocate(true);

    let mut output: Vec<Vec<f32>> =
        vec![Vec::with_capacity(chunk_output[0].len() * data[0].len() / chunk_size); data.len()];

    let mut data = data
        .iter()
        .map(|e| e.chunks_exact(chunk_size))
        .collect::<Vec<_>>();

    let mut buffer = Vec::with_capacity(data.len());

    loop {
        for chunk in &mut data {
            if let Some(chunk) = chunk.next() {
                buffer.push(chunk);
            } else {
                return Ok(output);
            }
        }

        let (_, frames) = cache
            .resampler
            .process_into_buffer(&buffer, &mut chunk_output, None)?;

        for (output, chunk) in output.iter_mut().zip(chunk_output.iter()) {
            output.extend_from_slice(&chunk[..frames]);
        }

        buffer.clear();
    }
}
