use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_f64(&mut self, value: f64) -> Result<(), MsgPackErr> {
        self.w.write_all(&[0xcb])?;
        self.w.write_all(&value.to_bits().to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn encode_f32(&mut self, value: f32) -> Result<(), MsgPackErr> {
        self.w.write_all(&[0xca])?;
        self.w.write_all(&value.to_bits().to_be_bytes())?;
        Ok(())
    }
}
