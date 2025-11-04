use crate::{decode::Decoder, error::MsgPackErr};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_str(&mut self, prefix: u8) -> Result<String, MsgPackErr> {
        let len = match prefix {
            0xa0..=0xbf => (prefix & 0x1f) as usize,
            0xd9 => self.read_u8()? as usize,
            0xda => self.read_u16()? as usize,
            0xdb => self.read_u32()? as usize,
            _ => return Err(MsgPackErr::InvalidFormat(prefix)),
        };

        let mut buf = vec![0; len];
        let _ = self.r.read_exact(&mut buf).map_err(MsgPackErr::Io);
        let s = String::from_utf8(buf).map_err(|_| MsgPackErr::InvalidUtf8)?;

        Ok(s)
    }
}
