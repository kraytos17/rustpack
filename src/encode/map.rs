use crate::{encode::Encoder, error::MsgPackErr, value::Value};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_map(&mut self, map: &[(Value, Value)]) -> Result<(), MsgPackErr> {
        let len = map.len();
        if len <= 15 {
            self.w.write_all(&[(0x80 | u8::try_from(len).unwrap())])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xde])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xdf])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        for (k, v) in map {
            self.encode(k)?;
            self.encode(v)?;
        }

        Ok(())
    }
}
