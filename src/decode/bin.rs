use crate::{decode::Decoder, error::MsgPackErr, value::Value};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_bin(&mut self, prefix: u8) -> Result<Value, MsgPackErr> {
        let len = match prefix {
            0xc4 => self.read_u8()? as usize,
            0xc5 => self.read_u16()? as usize,
            0xc6 => self.read_u32()? as usize,
            _ => return Err(MsgPackErr::InvalidFormat(prefix)),
        };

        let mut buf = vec![0u8; len];
        self.r.read_exact(&mut buf)?;

        Ok(Value::Binary(buf))
    }
}
