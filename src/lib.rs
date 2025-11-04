use crate::{decode::Decoder, encode::Encoder, error::MsgPackErr, value::Value};
use std::io::{Cursor, Read, Write};

mod decode;
mod encode;
mod error;
mod value;

/// Encode a `Value` into a `Vec<u8>`.
pub fn to_vec(value: &Value) -> Result<Vec<u8>, MsgPackErr> {
    let mut buf = Vec::new();
    {
        let mut enc = Encoder::new(&mut buf);
        enc.encode(value)?;
    }

    Ok(buf)
}

/// Encode a `Value` directly to a writer.
pub fn to_writer<W: Write>(writer: W, value: &Value) -> Result<(), MsgPackErr> {
    let mut enc = Encoder::new(writer);
    enc.encode(value)
}

/// Decode a `Value` from a byte slice.
pub fn from_slice(data: &[u8]) -> Result<Value, MsgPackErr> {
    let mut dec = Decoder::new(Cursor::new(data));
    dec.decode()
}

/// Decode a `Value` from a reader.
pub fn from_reader<R: Read>(reader: R) -> Result<Value, MsgPackErr> {
    let mut dec = Decoder::new(reader);
    dec.decode()
}
