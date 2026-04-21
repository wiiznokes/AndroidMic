use std::io::Write;

use android_mic::audio::player::process_audio;
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

fn bench_audio(c: &mut Criterion) {
    c.bench_function("audio_processing", |b| {
        let shared_buffer_size = 48000 * 2;

        let (mut producer, mut consumer) = RingBuffer::<u8>::new(shared_buffer_size);

        // pre-fill fake audio buffer
        let mut output = vec![0i16; 480 * 2];

        let source = make_random(48000);

        b.iter(|| {
            feed_producer(&mut producer, &source);

            process_audio::<i16>(&mut output, &mut consumer, 480);
        });
    });
}

criterion_group!(benches, bench_audio);
criterion_main!(benches);
