use crate::{decode::Decoder, error::MsgPackErr, value::Value};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_arr(&mut self, prefix: u8) -> Result<Value, MsgPackErr> {
        let len = match prefix {
            0x90..=0x9f => (prefix & 0x0f) as usize,
            0xdc => self.read_u16()? as usize,
            0xdd => self.read_u32()? as usize,
            _ => return Err(MsgPackErr::InvalidFormat(prefix)),
        };

        let mut arr = Vec::with_capacity(len);
        for _ in 0..len {
            let value = self.decode()?;
            arr.push(value);
        }

        Ok(Value::Array(arr))
    }
}
