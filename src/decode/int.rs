use crate::{
    decode::Decoder,
    error::MsgPackErr,
    value::{Integer, Value},
};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_int(&mut self, prefix: u8) -> Result<Value, MsgPackErr> {
        match prefix {
            0x00..=0x7f => Ok(Value::Integer(Integer::U64(prefix as u64))),
            0xe0..=0xff => Ok(Value::Integer(Integer::I64((prefix as i8) as i64))),
            0xcc => {
                let n = self.read_u8()? as u64;
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xcd => {
                let n = self.read_u16()? as u64;
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xce => {
                let n = self.read_u32()? as u64;
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xcf => {
                let n = self.read_u64()?;
                Ok(Value::Integer(Integer::U64(n)))
            }
            0xd0 => {
                let n = self.read_i8()? as i64;
                Ok(Value::Integer(Integer::I64(n)))
            }
            0xd1 => {
                let n = self.read_i16()? as i64;
                Ok(Value::Integer(Integer::I64(n)))
            }
            0xd2 => {
                let n = self.read_i32()? as i64;
                Ok(Value::Integer(Integer::I64(n)))
            }
            0xd3 => {
                let n = self.read_i64()?;
                Ok(Value::Integer(Integer::I64(n)))
            }
            _ => return Err(MsgPackErr::InvalidFormat(prefix)),
        }
    }
}
