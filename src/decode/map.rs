use crate::{decode::Decoder, error::MsgPackErr, value::Value};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_map(&mut self, prefix: u8) -> Result<Vec<(Value, Value)>, MsgPackErr> {
        let len = match prefix {
            0x80..=0x8f => (prefix & 0x0f) as usize,
            0xde => self.read_u16()? as usize,
            0xdf => self.read_u32()? as usize,
            _ => return Err(MsgPackErr::InvalidFormat(prefix)),
        };

        let mut map = Vec::with_capacity(len);
        for _ in 0..len {
            let key = self.decode()?;
            let val = self.decode()?;
            map.push((key, val));
        }

        Ok(map)
    }
}
