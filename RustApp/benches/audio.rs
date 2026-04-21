use std::io::Write;

use android_mic::{
    audio::{AudioPacketFormat, AudioProcessParams, player::process_audio},
    config::{AudioEffect, AudioFormat, ChannelCount, SampleRate},
    streamer::{AudioPacketMessage, AudioStream},
};
use criterion::{Criterion, criterion_group, criterion_main};
use rtrb::{Producer, RingBuffer};

fn make_random(size: usize) -> Vec<u8> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(42);

    let mut v = vec![0; size];

    rng.fill_bytes(&mut v);
    v
}

fn feed_producer(producer: &mut Producer<u8>, src: &[u8]) {
    producer.write(src).unwrap();
}

const SHARED_BUFFER_SIZE: usize = 48000 * 2;

fn bench_player(c: &mut Criterion) {
    c.bench_function("bench_player", |b| {
        let (mut producer, mut consumer) = RingBuffer::<u8>::new(SHARED_BUFFER_SIZE);

        // pre-fill fake audio buffer
        let mut output = vec![0i16; 480 * 2];

        let source = make_random(48000);

        b.iter(|| {
            feed_producer(&mut producer, &source);

            process_audio::<i16>(&mut output, &mut consumer, 480);
        });
    });
}

fn bench_process(c: &mut Criterion) {
    c.bench_function("bench_process", |b| {
        let (producer, mut consumer) = RingBuffer::<u8>::new(SHARED_BUFFER_SIZE);

        let audio_params = AudioProcessParams {
            target_format: AudioPacketFormat {
                sample_rate: SampleRate::S48000,
                audio_format: AudioFormat::I16,
                channel_count: ChannelCount::Mono,
            },
            denoise: None,
            amplify: None,
            post_effect: AudioEffect::NoEffect,
            speex_noise_suppress: -30,
            speex_vad_enabled: false,
            speex_vad_threshold: 80,
            speex_agc_enabled: false,
            speex_agc_target: 8000,
            speex_dereverb_enabled: false,
            speex_dereverb_level: 0.5,
        };

        let mut audio_stream = AudioStream::new(producer, audio_params, false);

        b.iter(|| {
            let source = make_random(3840);

            let packet = AudioPacketMessage {
                buffer: source,
                sample_rate: 48000,
                channel_count: 1,
                audio_format: 2,
            };

            audio_stream.process_audio_packet(packet).unwrap();

            let chunk = consumer.read_chunk(consumer.slots()).unwrap();
            chunk.commit_all();
        });
    });
}

criterion_group!(benches, bench_player, bench_process);
criterion_main!(benches);
