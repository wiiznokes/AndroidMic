use std::sync::{LazyLock, Mutex};

use nnnoiseless::DenoiseState;

struct DenoiseCache {
    sample_buffer: Vec<Vec<f32>>,
    denoisers: Vec<Box<DenoiseState<'static>>>,
}

static DENOISE_CACHE: LazyLock<Mutex<Option<DenoiseCache>>> = LazyLock::new(|| Mutex::new(None));

pub fn denoise_f32_stream(data: &[Vec<f32>]) -> anyhow::Result<Vec<Vec<f32>>> {
    let mut denoise_cache = DENOISE_CACHE.lock().unwrap();

    if denoise_cache.is_none() || data.len() != denoise_cache.as_ref().unwrap().denoisers.len() {
        *denoise_cache = Some(DenoiseCache {
            sample_buffer: vec![Vec::with_capacity(DenoiseState::FRAME_SIZE); data.len()],
            denoisers: vec![DenoiseState::new(); data.len()],
        });
    }

    let cache = denoise_cache.as_mut().unwrap();
    let mut output: Vec<Vec<f32>> = vec![Vec::new(); data.len()];
    let mut output_buffer_i16 = [0.0; DenoiseState::FRAME_SIZE];

    // Convert f32 to i16 range
    let data_i16: Vec<Vec<f32>> = data
        .iter()
        .map(|channel| channel.iter().map(|&x| (x * i16::MAX as f32)).collect())
        .collect();

    // Append new data into the cache
    for channel_idx in 0..data_i16.len() {
        cache.sample_buffer[channel_idx].extend_from_slice(&data_i16[channel_idx]);
    }

    while cache.sample_buffer[0].len() >= DenoiseState::FRAME_SIZE {
        for channel_idx in 0..data.len() {
            cache.denoisers[channel_idx].process_frame(
                &mut output_buffer_i16,
                &cache.sample_buffer[channel_idx][0..DenoiseState::FRAME_SIZE],
            );

            // Scale back to -1.0 to 1.0 range
            output[channel_idx].extend_from_slice(
                &output_buffer_i16
                    .iter()
                    .map(|&x| x / i16::MAX as f32)
                    .collect::<Vec<f32>>(),
            );
        }

        // Clear the sample buffer for the next round
        for channel in &mut cache.sample_buffer {
            channel.drain(0..DenoiseState::FRAME_SIZE);
        }
    }

    Ok(output)
}
