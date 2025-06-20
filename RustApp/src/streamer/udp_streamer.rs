use std::{io, net::IpAddr, time::Duration};

use futures::StreamExt;
use prost::Message;
use tokio::net::UdpSocket;
use tokio_util::{codec::LengthDelimitedCodec, udp::UdpFramed};

use crate::{
    audio::process::convert_audio_stream,
    streamer::{AudioPacketMessage, DEFAULT_PC_PORT, MAX_PORT, WriteError},
};

use super::{AudioPacketMessageOrdered, ConnectError, Status, StreamConfig, StreamerTrait};

const MAX_WAIT_TIME: Duration = Duration::from_millis(1500);

const DISCONNECT_LOOP_DETECTER_MAX: u32 = 1000;

pub struct UdpStreamer {
    ip: IpAddr,
    pub port: u16,
    stream_config: StreamConfig,
    state: UdpStreamerState,
}

#[allow(clippy::large_enum_variant)]
pub enum UdpStreamerState {
    Streaming {
        framed: UdpFramed<LengthDelimitedCodec>,
        tracked_sequence: u32,
    },
}

pub async fn new(ip: IpAddr, stream_config: StreamConfig) -> Result<UdpStreamer, ConnectError> {
    let mut socket = None;

    // try to always bind the same port, to not change it everytime Android side
    for p in DEFAULT_PC_PORT..=MAX_PORT {
        if let Ok(l) = UdpSocket::bind((ip, p)).await {
            socket = Some(l);
            break;
        }
    }

    let socket = if let Some(socket) = socket {
        socket
    } else {
        UdpSocket::bind((ip, 0))
            .await
            .map_err(ConnectError::CantBindPort)?
    };

    let addr = socket.local_addr().map_err(ConnectError::NoLocalAddress)?;

    let streamer = UdpStreamer {
        ip,
        port: addr.port(),
        stream_config,
        state: UdpStreamerState::Streaming {
            framed: UdpFramed::new(socket, LengthDelimitedCodec::new()),
            tracked_sequence: 0,
        },
    };

    Ok(streamer)
}

impl StreamerTrait for UdpStreamer {
    fn reconfigure_stream(&mut self, stream_config: StreamConfig) {
        self.stream_config = stream_config;
    }

    fn status(&self) -> Option<Status> {
        match &self.state {
            UdpStreamerState::Streaming { .. } => Some(Status::Listening {
                port: Some(self.port),
            }),
        }
    }

    async fn next(&mut self) -> Result<Option<Status>, ConnectError> {
        match &mut self.state {
            UdpStreamerState::Streaming {
                framed,
                tracked_sequence,
            } => {
                match framed.next().await {
                    Some(Ok((frame, addr))) => {
                        let mut res = None;

                        match AudioPacketMessageOrdered::decode(frame) {
                            Ok(packet) => {
                                if packet.sequence_number < *tracked_sequence {
                                    // drop packet
                                    info!(
                                        "dropped packet: old sequence number {} < {}",
                                        packet.sequence_number, tracked_sequence
                                    );
                                }
                                *tracked_sequence = packet.sequence_number;

                                let packet = packet.audio_packet.unwrap();
                                let buffer_size = packet.buffer.len();

                                let audio_params = self.stream_config.to_audio_params();
                                if let Ok(buffer) = convert_audio_stream(
                                    &mut self.stream_config.buff,
                                    packet,
                                    audio_params,
                                ) {
                                    // compute the audio wave from the buffer
                                    res = Some(Status::UpdateAudioWave {
                                        data: AudioPacketMessage::to_wave_data(&buffer),
                                    });

                                    debug!("From {:?}, received {} bytes", addr, buffer_size);
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
                    None => {
                        todo!()
                    }
                }
            }
        }
    }
}
