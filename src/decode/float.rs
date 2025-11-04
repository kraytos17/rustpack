use crate::{decode::Decoder, error::MsgPackErr, value::Value};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_float(&mut self, prefix: u8) -> Result<Value, MsgPackErr> {
        match prefix {
            0xca => {
                let bits = self.read_u32()?;
                let f = f32::from_bits(bits);
                Ok(Value::Float(f as f64))
            }
            0xcb => {
                let bits = self.read_u64()?;
                let f = f64::from_bits(bits);
                Ok(Value::Float(f))
            }
            _ => Err(MsgPackErr::InvalidFormat(prefix)),
        }
    }
}
