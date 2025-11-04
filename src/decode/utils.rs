use crate::{decode::Decoder, error::MsgPackErr};
use std::io::Read;

impl<R: Read> Decoder<R> {
    #[inline]
    pub(crate) fn read_u8(&mut self) -> Result<u8, MsgPackErr> {
        let mut buf = [0u8; 1];
        self.r.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    pub(crate) fn read_u16(&mut self) -> Result<u16, MsgPackErr> {
        let mut buf = [0u8; 2];
        self.r.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    #[inline]
    pub(crate) fn read_u32(&mut self) -> Result<u32, MsgPackErr> {
        let mut buf = [0u8; 4];
        self.r.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    #[inline]
    pub(crate) fn read_u64(&mut self) -> Result<u64, MsgPackErr> {
        let mut buf = [0u8; 8];
        self.r.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }

    #[inline]
    pub(crate) fn read_i8(&mut self) -> Result<i8, MsgPackErr> {
        Ok(self.read_u8()? as i8)
    }

    #[inline]
    pub(crate) fn read_i16(&mut self) -> Result<i16, MsgPackErr> {
        let mut buf = [0u8; 2];
        self.r.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    #[inline]
    pub(crate) fn read_i32(&mut self) -> Result<i32, MsgPackErr> {
        let mut buf = [0u8; 4];
        self.r.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    #[inline]
    pub(crate) fn read_i64(&mut self) -> Result<i64, MsgPackErr> {
        let mut buf = [0u8; 8];
        self.r.read_exact(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }

    #[inline]
    pub(crate) fn read_f32(&mut self) -> Result<f32, MsgPackErr> {
        let mut buf = [0u8; 4];
        self.r.read_exact(&mut buf)?;
        Ok(f32::from_bits(u32::from_be_bytes(buf)))
    }

    #[inline]
    pub(crate) fn read_f64(&mut self) -> Result<f64, MsgPackErr> {
        let mut buf = [0u8; 8];
        self.r.read_exact(&mut buf)?;
        Ok(f64::from_bits(u64::from_be_bytes(buf)))
    }
}
