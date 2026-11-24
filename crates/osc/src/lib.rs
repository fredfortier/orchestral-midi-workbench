//! OSC wire format and typed command-line arguments for `DrivenByMoss`.

use std::io;

pub use rosc::{OscMessage, OscPacket, OscType, address::verify_address};

/// Parse one typed command argument.
/// # Errors
/// Rejects unsupported prefixes, invalid numbers, and embedded NUL characters.
pub fn parse_argument(argument: &str) -> Result<OscType, String> {
    match argument.split_once(':') {
        Some(("i", value)) => value
            .parse()
            .map(OscType::Int)
            .map_err(|error| format!("invalid int32 {value:?}: {error}")),
        Some(("f", value)) => {
            let number: f32 = value
                .parse()
                .map_err(|error| format!("invalid float32 {value:?}: {error}"))?;
            if number.is_finite() {
                Ok(OscType::Float(number))
            } else {
                Err(format!("float32 must be finite: {value:?}"))
            }
        }
        Some(("s", value)) if !value.contains('\0') => Ok(OscType::String(value.into())),
        _ => Err("expected i:INT, f:FLOAT, or s:STRING without NUL characters".into()),
    }
}

/// Encode one OSC message addressed to the extension.
/// # Errors
/// Rejects invalid addresses, arguments, and payloads too large for IPv4 UDP.
pub fn encode_command(address: String, arguments: &[String]) -> Result<Vec<u8>, String> {
    verify_address(&address).map_err(|error| format!("invalid OSC address: {error}"))?;
    let args = arguments
        .iter()
        .map(|arg| parse_argument(arg))
        .collect::<Result<_, _>>()?;
    let packet = OscPacket::Message(OscMessage {
        addr: address,
        args,
    });
    let bytes = rosc::encoder::encode(&packet).map_err(|error| error.to_string())?;
    if bytes.len() > 65_507 {
        return Err("OSC command exceeds the IPv4 UDP payload limit".into());
    }
    Ok(bytes)
}

/// Decode one complete OSC datagram.
/// # Errors
/// Rejects malformed packets and trailing bytes.
pub fn decode_datagram(bytes: &[u8]) -> io::Result<OscPacket> {
    let (remaining, packet) = rosc::decoder::decode_udp(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if !remaining.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing bytes in OSC datagram",
        ));
    }
    Ok(packet)
}
