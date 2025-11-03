use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_str(&mut self, s: &str) -> Result<(), MsgPackErr> {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if len <= 31 {
            self.w.write_all(&[(0xa0 | u8::try_from(len).unwrap())])?;
        } else if u8::try_from(len).is_ok() {
            self.w.write_all(&[0xd9])?;
            self.w.write_all(&[u8::try_from(len).unwrap()])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xda])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xdb])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        self.w.write_all(bytes)?;
        Ok(())
    }
}
