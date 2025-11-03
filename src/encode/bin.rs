use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_bin(&mut self, bytes: &[u8]) -> Result<(), MsgPackErr> {
        let len = bytes.len();
        if u8::try_from(len).is_ok() {
            self.w.write_all(&[0xc4])?;
            self.w.write_all(&[u8::try_from(len).unwrap()])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xc5])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xc6])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        self.w.write_all(bytes)?;
        Ok(())
    }
}
