use std::{
    error::Error,
    net::{SocketAddr, UdpSocket},
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use rosc::{OscMessage, OscPacket, OscType, address::verify_address, decoder, encoder};

#[derive(Parser)]
#[command(about = "Send and inspect DrivenByMoss OSC messages over UDP")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print incoming OSC messages and bundles until interrupted.
    Listen {
        /// Local IP and port matching the extension's feedback destination.
        bind: SocketAddr,
    },
    /// Send one OSC message to the extension's command port.
    Send {
        /// IP and port of the OSC extension.
        target: SocketAddr,
        /// OSC address, for example /refresh.
        address: String,
        /// Optional arguments with i: (int32), f: (float32), or s: (string) prefixes.
        #[arg(value_parser = parse_argument)]
        arguments: Vec<OscType>,
    },
}

fn parse_argument(argument: &str) -> Result<OscType, String> {
    match argument.split_once(':') {
        Some(("i", value)) => value
            .parse()
            .map(OscType::Int)
            .map_err(|error| format!("invalid int32 {value:?}: {error}")),
        Some(("f", value)) => value
            .parse()
            .map(OscType::Float)
            .map_err(|error| format!("invalid float32 {value:?}: {error}")),
        Some(("s", value)) => Ok(OscType::String(value.into())),
        _ => Err("expected i:INT, f:FLOAT, or s:STRING".into()),
    }
}

fn listen(bind: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind(bind).map_err(|error| format!("bind {bind}: {error}"))?;
    eprintln!("Listening on {}", socket.local_addr()?);
    let mut buffer = vec![0; 65_536];
    loop {
        let (size, peer) = socket.recv_from(&mut buffer)?;
        match decoder::decode_udp(&buffer[..size]) {
            Ok(([], packet)) => println!("{peer} {packet:?}"),
            Ok(_) => eprintln!("{peer} ignored OSC datagram with trailing bytes"),
            Err(error) => eprintln!("{peer} ignored invalid OSC datagram: {error}"),
        }
    }
}

fn send(target: SocketAddr, addr: String, args: Vec<OscType>) -> Result<(), Box<dyn Error>> {
    verify_address(&addr)?;
    let bytes = encoder::encode(&OscPacket::Message(OscMessage { addr, args }))?;
    let bind = if target.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind)?;
    socket
        .send_to(&bytes, target)
        .map_err(|error| format!("send to {target}: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Listen { bind } => listen(bind),
        Command::Send {
            target,
            address,
            arguments,
        } => send(target, address, arguments),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
