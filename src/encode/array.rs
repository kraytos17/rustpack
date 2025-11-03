use crate::{encode::Encoder, error::MsgPackErr, value::Value};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_arr(&mut self, arr: &[Value]) -> Result<(), MsgPackErr> {
        let len = arr.len();
        if len <= 15 {
            self.w.write_all(&[(0x90 | u8::try_from(len).unwrap())])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xdc])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xdd])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        for v in arr {
            let _ = self.encode(v);
        }

        Ok(())
    }
}
