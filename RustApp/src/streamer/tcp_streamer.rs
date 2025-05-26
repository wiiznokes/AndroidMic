use std::{io, net::IpAddr, time::Duration};

use futures::StreamExt;
use prost::Message;
use rtrb::Producer;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    audio::{resampler::convert_audio_stream, AudioPacketFormat},
    config::AudioFormat,
    streamer::{WriteError, DEFAULT_PC_PORT, MAX_PORT},
};

use super::{AudioPacketMessage, ConnectError, Status, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

pub struct TcpStreamer {
    ip: IpAddr,
    pub port: u16,
    producer: Producer<u8>,
    format: AudioPacketFormat,
    pub state: TcpStreamerState,
    pub nnoise: Box<nnnoiseless::DenoiseState<'static>>,
}

#[allow(clippy::large_enum_variant)]
pub enum TcpStreamerState {
    Listening {
        listener: TcpListener,
    },
    Streaming {
        framed: Framed<TcpStream, LengthDelimitedCodec>,
        disconnect_loop_detecter: u32,
    },
}

pub async fn new(
    ip: IpAddr,
    producer: Producer<u8>,
    format: AudioPacketFormat,
) -> Result<TcpStreamer, ConnectError> {
    let mut listener = None;

    // try to always bind the same port, to not change it everytime Android side
    for p in DEFAULT_PC_PORT..=MAX_PORT {
        if let Ok(l) = TcpListener::bind((ip, p)).await {
            listener = Some(l);
            break;
        }
    }

    let listener = if let Some(listener) = listener {
        listener
    } else {
        TcpListener::bind((ip, 0))
            .await
            .map_err(ConnectError::CantBindPort)?
    };

    let addr = TcpListener::local_addr(&listener).map_err(ConnectError::NoLocalAddress)?;

    let streamer = TcpStreamer {
        ip,
        port: addr.port(),
        producer,
        format,
        state: TcpStreamerState::Listening { listener },
        nnoise: nnnoiseless::DenoiseState::new(),
    };

    Ok(streamer)
}

impl StreamerTrait for TcpStreamer {
    fn set_buff(&mut self, producer: Producer<u8>, format: AudioPacketFormat) {
        self.producer = producer;
        self.format = format;
    }

    fn status(&self) -> Option<Status> {
        match &self.state {
            TcpStreamerState::Listening { .. } => Some(Status::Listening {
                port: Some(self.port),
            }),
            TcpStreamerState::Streaming { .. } => Some(Status::Connected),
        }
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        match &mut self.state {
            TcpStreamerState::Listening { listener } => {
                let addr =
                    TcpListener::local_addr(listener).map_err(ConnectError::NoLocalAddress)?;

                info!("TCP server listening on {}", addr);

                let (stream, addr) = listener.accept().await.map_err(ConnectError::CantAccept)?;

                info!("connection accepted, remote address: {}", addr);

                self.state = TcpStreamerState::Streaming {
                    framed: Framed::new(stream, LengthDelimitedCodec::new()),
                    disconnect_loop_detecter: 0,
                };

                Ok(Some(Status::Connected))
            }
            TcpStreamerState::Streaming {
                framed,
                disconnect_loop_detecter: _,
            } => {
                match framed.next().await {
                    Some(Ok(frame)) => {
                        let mut res = None;

                        match AudioPacketMessage::decode(frame) {
                            Ok(packet) => {
                                let buffer_size = packet.buffer.len();
                                let chunk_size = std::cmp::min(buffer_size, self.producer.slots());

                                // mapping from android AudioFormat to encoding size
                                let audio_format =
                                    AudioFormat::from_android_format(packet.audio_format).unwrap();
                                let encoding_size =
                                    audio_format.sample_size() * packet.channel_count as usize;

                                // make sure chunk_size is a multiple of encoding_size
                                let correction = chunk_size % encoding_size;

                                match self.producer.write_chunk_uninit(chunk_size - correction) {
                                    Ok(chunk) => {
                                        let mut final_res: Vec<u8> = Vec::new();

                                        packet
                                            .buffer
                                            .chunks(nnnoiseless::DenoiseState::FRAME_SIZE * 2)
                                            .for_each(|e| {

                                                if e.len() != nnnoiseless::DenoiseState::FRAME_SIZE * 2 {
                                                    return;
                                                }
                                                
                                                let mut res = vec![0f32; nnnoiseless::DenoiseState::FRAME_SIZE];

                                                use crate::audio::AudioBytes;

                                                let input = e
                                                    .chunks(2)
                                                    .map(|slice| {
                                                        i16::from_bytes(slice).unwrap() as f32
                                                    })
                                                    .collect::<Vec<_>>();

                                                self.nnoise.process_frame(&mut res, &input);

                                                let mut res: Vec<u8> = res
                                                    .into_iter()
                                                    .map(|v| {
                                                        let v = v as i16;
                                                        v.to_le_bytes()
                                                    })
                                                    .flatten()
                                                    .collect();

                                                final_res.append(&mut res);
                                            });

                                        info!(
                                            "FRAME_SIZE = {}",
                                            nnnoiseless::DenoiseState::FRAME_SIZE
                                        );
                                        info!("buffer_size = {}", buffer_size);

                                        chunk.fill_from_iter(final_res.into_iter());
                                        debug!(
                                            "received {} bytes, corrected {} bytes, lost {} bytes",
                                            buffer_size,
                                            correction,
                                            buffer_size - chunk_size + correction
                                        );
                                    }
                                    Err(e) => {
                                        warn!("dropped packet: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(ConnectError::WriteError(WriteError::Deserializer(e)));
                            }
                        }

                        Ok(res)
                    }

                    Some(Err(e)) => {
                        match e.kind() {
                            io::ErrorKind::TimedOut => Ok(None), // timeout use to check for input on stdin
                            io::ErrorKind::WouldBlock => Ok(None), // trigger on Linux when there is no stream input
                            _ => Err(WriteError::Io(e))?,
                        }
                    }
                    None => Err(ConnectError::Disconnected),
                }
            }
        }
    }
}
