use std::{io, sync::mpsc::Sender, thread};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use strum::{Display, EnumString};

pub enum UserAction {
    Quit,
}

#[allow(clippy::single_match)]
pub fn start_listening(tx: Sender<UserAction>) {
    let _join = thread::spawn(move || loop {
        match event::read() {
            Ok(event) => match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => tx.send(UserAction::Quit).unwrap(),
                _ => {}
            },
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    });
}

pub fn print_avaible_action() {
    println!();
    println!("Available options:");
    println!("quit: q");
    println!();
}

#[derive(Parser, Debug)]
#[clap(author = "wiiznokes", version, about = "AndroidMic", long_about = None)]
pub struct Args {
    #[arg(short, long, help = "example: -i 192.168.1.79")]
    pub ip: Option<String>,

    #[arg(short = 'm', long = "mode", id = "connection mode", help = "UDP or TCP", default_value_t = ConnectionMode::Udp)]
    pub connection_mode: ConnectionMode,

    #[arg(short = 'f', long = "format", id = "audio format",  help = "i16 or i32", default_value_t = AudioFormat::I16)]
    pub audio_format: AudioFormat,

    #[arg(
        short = 'o',
        long = "output",
        id = "output device",
        default_value_t = 0
    )]
    pub output_device: usize,

    /// should not have default config because it depend on the divice
    #[arg(short = 'c', long = "channel", id = "channel count", help = "1 or 2")]
    pub channel_count: Option<ChannelCount>,

    /// should not have default config because it depend on the divice
    #[arg(short = 'r', long = "sample", id = "sample rate")]
    pub sample_rate: Option<u32>,
}

// todo: parse it
pub fn ask_ip() -> String {
    println!("Please enter the ip of the host (The IP of your PC)");
    println!("Help: something like: 192.168.1.79");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().into()
}

#[derive(Debug, Clone, EnumString, PartialEq, Display)]
pub enum ConnectionMode {
    #[strum(serialize = "udp", serialize = "UDP")]
    Udp,
    #[strum(serialize = "tcp", serialize = "TCP")]
    Tcp,
}

#[derive(Debug, Clone, EnumString, PartialEq, Display)]
pub enum ChannelCount {
    #[strum(serialize = "mono", serialize = "MONO", serialize = "1")]
    Mono,
    #[strum(serialize = "stereo", serialize = "STEREO", serialize = "2")]
    Stereo,
}

#[derive(Debug, Clone, EnumString, PartialEq, Display)]
pub enum AudioFormat {
    #[strum(serialize = "i16")]
    I16,
    #[strum(serialize = "i32")]
    I32,
}
