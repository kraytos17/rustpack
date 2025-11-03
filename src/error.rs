use std::{fmt, io};

#[derive(Debug)]
pub enum MsgPackErr {
    UnexpectedEof,
    InvalidFormat(u8),
    InvalidUtf8,
    TypeMismatch,
    Io(io::Error),
}

impl From<io::Error> for MsgPackErr {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl fmt::Display for MsgPackErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::InvalidFormat(b) => write!(f, "invalid format byte: {b:#x}"),
            Self::InvalidUtf8 => write!(f, "invalid utf-8 in string"),
            Self::TypeMismatch => write!(f, "type mismatch"),
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for MsgPackErr {}
