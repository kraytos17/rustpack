use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_i64(&mut self, value: i64) -> Result<(), MsgPackErr> {
        if (0..=127).contains(&value) {
            self.w.write_all(&[u8::try_from(value).unwrap()])?;
        } else if (-32..=-1).contains(&value) {
            self.w
                .write_all(&[0xe0u8.wrapping_add(u8::try_from(value + 32).unwrap())])?;
        } else if (-128..=-33).contains(&value) {
            self.w.write_all(&[0xd0, u8::try_from(value).unwrap()])?;
        } else if (-32_768..=-129).contains(&value) {
            self.w.write_all(&[0xd1])?;
            self.w
                .write_all(&i16::try_from(value).unwrap().to_be_bytes())?;
        } else if (-2_147_483_648..=-32_769).contains(&value) {
            self.w.write_all(&[0xd2])?;
            self.w
                .write_all(&i32::try_from(value).unwrap().to_be_bytes())?;
        } else if value < 0 {
            self.w.write_all(&[0xd3])?;
            self.w.write_all(&value.to_be_bytes())?;
        } else if (128..=255).contains(&value) {
            self.w.write_all(&[0xcc, u8::try_from(value).unwrap()])?;
        } else if (256..=65_535).contains(&value) {
            self.w.write_all(&[0xcd])?;
            self.w
                .write_all(&u16::try_from(value).unwrap().to_be_bytes())?;
        } else if (65_536..=4_294_967_295).contains(&value) {
            self.w.write_all(&[0xce])?;
            self.w
                .write_all(&u32::try_from(value).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xcf])?;
            self.w
                .write_all(&u64::try_from(value).unwrap().to_be_bytes())?;
        }

        Ok(())
    }

    pub(crate) fn encode_u64(&mut self, value: u64) -> Result<(), MsgPackErr> {
        if value <= 127 {
            self.w.write_all(&[u8::try_from(value).unwrap()])?;
        } else if value <= 0xff {
            self.w.write_all(&[0xcc, u8::try_from(value).unwrap()])?;
        } else if value <= 0xffff {
            self.w.write_all(&[0xcd])?;
            self.w
                .write_all(&u16::try_from(value).unwrap().to_be_bytes())?;
        } else if value <= 0xffff_ffff {
            self.w.write_all(&[0xce])?;
            self.w
                .write_all(&u32::try_from(value).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xcf])?;
            self.w.write_all(&value.to_be_bytes())?;
        }

        Ok(())
    }
}
