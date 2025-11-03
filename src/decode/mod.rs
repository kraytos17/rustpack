use crate::{error::MsgPackErr, value::Value};
use std::io::Read;

pub struct Decoder<R: Read> {
    pub(crate) r: R,
}

impl<R: Read> Decoder<R> {
    pub const fn new(r: R) -> Self {
        Self { r }
    }
}
