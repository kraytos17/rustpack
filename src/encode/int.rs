use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_i64(&mut self, value: i64) -> Result<(), MsgPackErr> {
        if (0..=127).contains(&value) {
            self.w.write_all(&[value as u8])?;
        } else if (-32..=-1).contains(&value) {
            self.w
                .write_all(&[(0xe0u8).wrapping_add((value + 32) as u8)])?;
        } else if (-128..=-33).contains(&value) {
            self.w.write_all(&[0xd0, value as i8 as u8])?;
        } else if (-32_768..=-129).contains(&value) {
            self.w.write_all(&[0xd1])?;
            self.w.write_all(&(value as i16).to_be_bytes())?;
        } else if (-2_147_483_648..=-32_769).contains(&value) {
            self.w.write_all(&[0xd2])?;
            self.w.write_all(&(value as i32).to_be_bytes())?;
        } else if value < 0 {
            self.w.write_all(&[0xd3])?;
            self.w.write_all(&value.to_be_bytes())?;
        } else if (128..=255).contains(&value) {
            self.w.write_all(&[0xcc, value as u8])?;
        } else if (256..=65_535).contains(&value) {
            self.w.write_all(&[0xcd])?;
            self.w.write_all(&(value as u16).to_be_bytes())?;
        } else if (65_536..=4_294_967_295).contains(&value) {
            self.w.write_all(&[0xce])?;
            self.w.write_all(&(value as u32).to_be_bytes())?;
        } else {
            self.w.write_all(&[0xcf])?;
            self.w.write_all(&(value as u64).to_be_bytes())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn encode_i64_to_vec(val: i64) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_i64(val).unwrap();
        buf
    }

    fn encode_u64_to_vec(val: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_u64(val).unwrap();
        buf
    }

    #[test]
    fn test_encode_fixint_pos_neg() {
        assert_eq!(encode_i64_to_vec(-1), vec![0xff]);
        assert_eq!(encode_i64_to_vec(0), vec![0x00]);
        assert_eq!(encode_i64_to_vec(127), vec![0x7f]);
        assert_eq!(encode_i64_to_vec(-32), vec![0xe0]);
        assert_eq!(encode_i64_to_vec(-33), vec![0xd0, 0xdf]);
    }

    #[test]
    fn test_encode_i64_boundaries() {
        assert_eq!(encode_i64_to_vec(-128), vec![0xd0, 0x80]);
        assert_eq!(encode_i64_to_vec(-129), vec![0xd1, 0xff, 0x7f]);
        assert_eq!(encode_i64_to_vec(-32768), vec![0xd1, 0x80, 0x00]);
        assert_eq!(
            encode_i64_to_vec(-32769),
            vec![0xd2, 0xff, 0xff, 0x7f, 0xff]
        );
        assert_eq!(
            encode_i64_to_vec(-2147483648),
            vec![0xd2, 0x80, 0x00, 0x00, 0x00]
        );
        assert_eq!(encode_i64_to_vec(i64::MIN), {
            let mut v = vec![0xd3];
            v.extend_from_slice(&i64::MIN.to_be_bytes());
            v
        });
    }

    #[test]
    fn test_encode_u64_boundaries() {
        assert_eq!(encode_u64_to_vec(127), vec![0x7f]);
        assert_eq!(encode_u64_to_vec(128), vec![0xcc, 0x80]);
        assert_eq!(encode_u64_to_vec(255), vec![0xcc, 0xff]);
        assert_eq!(encode_u64_to_vec(256), vec![0xcd, 0x01, 0x00]);
        assert_eq!(encode_u64_to_vec(65535), vec![0xcd, 0xff, 0xff]);
        assert_eq!(encode_u64_to_vec(65536), vec![0xce, 0x00, 0x01, 0x00, 0x00]);
        assert_eq!(
            encode_u64_to_vec(4_294_967_295),
            vec![0xce, 0xff, 0xff, 0xff, 0xff]
        );
        assert_eq!(encode_u64_to_vec(4_294_967_296), {
            let mut v = vec![0xcf];
            v.extend_from_slice(&4_294_967_296u64.to_be_bytes());
            v
        });
    }
}
