use crate::{
    error::MsgPackErr,
    value::{Integer, Value},
};
use std::io::Write;

mod array;
mod bin;
mod ext;
mod float;
mod int;
mod map;
mod str;

pub struct Encoder<W: Write> {
    pub(crate) w: W,
}

impl<W: Write> Encoder<W> {
    pub const fn new(w: W) -> Self {
        Self { w }
    }

    pub fn encode(&mut self, val: &Value) -> Result<(), MsgPackErr> {
        match val {
            Value::Nil => self.w.write_all(&[0xc0])?,
            Value::Boolean(b) => self.w.write_all(&[if *b { 0xc3 } else { 0xc2 }])?,
            Value::Integer(i) => match i {
                Integer::U64(v) => self.encode_u64(*v)?,
                Integer::I64(v) => self.encode_i64(*v)?,
            },
            Value::Float(f) => self.encode_f64(*f)?,
            Value::String(s) => self.encode_str(s)?,
            Value::Binary(bin) => self.encode_bin(bin)?,
            Value::Array(arr) => self.encode_arr(arr)?,
            Value::Map(m) => self.encode_map(m)?,
            Value::Extension(e) => self.encode_ext(e)?,
        }

        Ok(())
    }
}
