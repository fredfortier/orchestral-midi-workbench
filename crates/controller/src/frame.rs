//! Plain JSON frame bodies. The socket's length prefix bounds each complete message.
use std::io;
pub const MAX_ENCODED_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DECODED_BYTES: usize = MAX_ENCODED_BYTES;
/// # Errors
/// Refuses an empty or oversized message; never compresses or truncates it.
pub fn encode(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    if bytes.is_empty() || bytes.len() > MAX_ENCODED_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "JSON message exceeds socket limit",
        ));
    }
    Ok(bytes)
}
/// # Errors
/// Refuses an empty or oversized message.
pub fn decode(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    encode(bytes)
}
