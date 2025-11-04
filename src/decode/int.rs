use crate::{
    decode::Decoder,
    error::MsgPackErr,
    value::{Integer, Value},
};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_int(&mut self, prefix: u8) -> Result<Value, MsgPackErr> {
        match prefix {
            0x00..=0x7f => Ok(Value::Integer(Integer::U64(u64::from(prefix)))),
            0xe0..=0xff => Ok(Value::Integer(Integer::I64(i64::from(prefix as i8)))),
            0xcc => {
                let n = u64::from(self.read_u8()?);
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xcd => {
                let n = u64::from(self.read_u16()?);
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xce => {
                let n = u64::from(self.read_u32()?);
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xcf => {
                let n = self.read_u64()?;
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xd0 => {
                let n = i64::from(self.read_i8()?);
                Ok(Value::Integer(Integer::I64(n)))
            }
            0xd1 => {
                let n = i64::from(self.read_i16()?);
                Ok(Value::Integer(Integer::I64(n)))
            }
            0xd2 => {
                let n = i64::from(self.read_i32()?);
                Ok(Value::Integer(Integer::I64(n)))
            }
            0xd3 => {
                let n = self.read_i64()?;
                Ok(Value::Integer(Integer::I64(n)))
            }
            _ => Err(MsgPackErr::InvalidFormat(prefix)),
        }
    }
}
