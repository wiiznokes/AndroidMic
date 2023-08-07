use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError,
};
use rtrb::{chunks::ChunkError, Consumer, Producer, RingBuffer};
use std::{
    io::{self},
    net::UdpSocket,
};
use user_action::UserAction;

use std::sync::mpsc;

mod user_action;

struct App {
    audio_player: Option<cpal::Stream>,
}

impl App {
    fn new() -> Self {
        App { audio_player: None }
    }
}

fn main() {
    let mut app = App::new();

    let capacity = 5 * 1024;
    // Buffer to store received data
    let (mut producer, consumer) = RingBuffer::<u8>::new(capacity);

    match setup_audio(consumer) {
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
        Ok(steam) => match steam.play() {
            Ok(_) => app.audio_player = Some(steam),
            Err(e) => {
                eprintln!("{:?}", e);
                return;
            }
        },
    }

    let bind_port = 55555;
    let socket = UdpSocket::bind(("0.0.0.0", bind_port)).expect("Failed to bind to socket");

    let udp_buf = [0u8; 1024];

    let (tx, rx) = mpsc::channel::<UserAction>();
    user_action::start_listening(tx);
    user_action::print_avaible_action();

    let mut iteration: f64 = 0.0;
    let mut item_lossed: f64 = 0.0;
    let mut item_moved: f64 = 0.0;

    use std::time::Instant;
    let now = Instant::now();
    loop {
        if let Ok(action) = rx.try_recv() {
            match action {
                UserAction::Quit => {
                    println!("quit requested");
                    break;
                }
            }
        }

        match write_to_buff(&socket, &mut producer, udp_buf) {
            Ok(moved) => item_moved += moved as f64,
            Err(e) => match e {
                WriteError::Udp(e) => {
                    eprintln!("{:?}", e);
                    break;
                }
                WriteError::BufferOverfilled(moved, lossed) => {
                    item_lossed += lossed as f64;
                    item_moved += moved as f64;
                }
            },
        }
        iteration += 1.0;
    }

    println!();
    println!("Stats:");
    println!("elapsed: {:.2?}", now.elapsed());
    println!(
        "iteration: {}, item moved: {}, item lossed: {}",
        iteration, item_moved, item_lossed
    );
    println!("moved by iteration: {}", item_moved / iteration);
    println!("lossed by iteration: {}", item_lossed / iteration);
    println!(
        "success: {}%",
        (item_moved / (item_lossed + item_moved)) * 100.0
    )
}

fn setup_audio(mut consumer: Consumer<u8>) -> Result<cpal::Stream, BuildStreamError> {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();

    // let config = StreamConfig{
    //     channels: 1,
    //     sample_rate: SampleRate(16000),
    //     buffer_size: BufferSize::Default(1024),
    // };
    let config: cpal::StreamConfig = device.default_output_config().unwrap().into();

    println!();
    println!("Audio config:");
    println!("- number of channel: {}", config.channels);
    println!("- sample rate: {}", config.sample_rate.0);
    println!("- buffer size: {:?}", config.buffer_size);
    println!();

    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            match consumer.read_chunk(data.len()) {
                Ok(chunk) => {
                    //println!("read_chunk {}", chunk.len());
                    let mut iter = chunk.into_iter();
                    // a frame has 480 samples
                    for frame in data.chunks_mut(channels) {
                        let Some(byte1) = iter.next()  else {
                            // should not happend, because we read data.len()
                            // chunk, but happend sometime
                            //eprintln!("None next byte1");
                            return;
                        };
                        let Some(byte2) = iter.next()  else {
                            //eprintln!("None next byte2, loose byte1");
                            return;
                        };

                        // Combine the two u8 values into a single i16
                        // don't ask me why we inverse bytes here (probably because of Endian stuff)
                        let result_i16: i16 = (byte2 as i16) << 8 | byte1 as i16;

                        // cursor method (should work on more PC but less optimize i think)
                        // let mut cursor: Cursor<Vec<u8>> = Cursor::new(vec![byte1, byte2]);
                        // let result_i16 = cursor.read_i16::<LittleEndian>().unwrap();

                        // a sample has two cases (probably left/right)
                        for sample in frame.iter_mut() {
                            *sample = result_i16;
                        }
                    }
                }
                Err(ChunkError::TooFewSlots(available_slots)) => {
                    let mut iter = consumer.read_chunk(available_slots).unwrap().into_iter();
                    for frame in data.chunks_mut(channels) {
                        let Some(byte1) = iter.next()  else {
                            //eprintln!("None next byte1");
                            return;
                        };
                        let Some(byte2) = iter.next()  else {
                            //eprintln!("None next byte2, loose byte1");
                            return;
                        };
                        let result_i16: i16 = (byte2 as i16) << 8 | byte1 as i16;
                        for sample in frame.iter_mut() {
                            *sample = result_i16;
                        }
                    }
                }
            }
        },
        err_fn,
        None, // todo: try timeout
    )
}

enum WriteError {
    Udp(io::Error),
    BufferOverfilled(usize, usize), // moved, lossed
}

/// return the number of item moved
/// or an error
fn write_to_buff(
    socket: &UdpSocket,
    producer: &mut Producer<u8>,
    mut tmp_buf: [u8; 1024],
) -> Result<usize, WriteError> {
    // Receive data into the buffer
    match socket.recv_from(&mut tmp_buf) {
        Ok((size, _)) => match producer.write_chunk_uninit(size) {
            Ok(chunk) => {
                let moved_item = chunk.fill_from_iter(tmp_buf);
                if moved_item == size {
                    Ok(size)
                } else {
                    Err(WriteError::BufferOverfilled(moved_item, size - moved_item))
                }
            }
            Err(ChunkError::TooFewSlots(remaining_slots)) => {
                let chunk = producer.write_chunk_uninit(remaining_slots).unwrap();

                let moved_item = chunk.fill_from_iter(tmp_buf);

                Err(WriteError::BufferOverfilled(moved_item, size - moved_item))
            }
        },
        Err(e) => Err(WriteError::Udp(e)),
    }
}
