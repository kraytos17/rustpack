use crate::{error::MsgPackErr, value::Value};
use std::io::Read;

mod array;
mod bin;
mod ext;
mod float;
mod int;
mod map;
mod str;
mod utils;

pub struct Decoder<R: Read> {
    pub(crate) r: R,
}

impl<R: Read> Decoder<R> {
    pub const fn new(r: R) -> Self {
        Self { r }
    }

    pub fn decode(&mut self) -> Result<Value, MsgPackErr> {
        let prefix = self.read_u8()?;
        match prefix {
            0xc0 => Ok(Value::Nil),
            0xc2 => Ok(Value::Boolean(false)),
            0xc3 => Ok(Value::Boolean(true)),
            0x00..=0x7f | 0xe0..=0xff | 0xcc..=0xd3 => self.decode_int(prefix),
            0xca | 0xcb => self.decode_float(prefix),
            0xa0..=0xbf | 0xd9..=0xdb => self.decode_str(prefix),
            0xc4..=0xc6 => self.decode_bin(prefix),
            0x90..=0x9f | 0xdc | 0xdd => self.decode_arr(prefix),
            0x80..=0x8f | 0xde | 0xdf => self.decode_map(prefix),
            0xc7..=0xc9 | 0xd4..=0xd8 => self.decode_ext(prefix),
            _ => Err(MsgPackErr::InvalidFormat(prefix)),
        }
    }
}
