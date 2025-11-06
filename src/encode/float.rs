use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_f64(&mut self, value: f64) -> Result<(), MsgPackErr> {
        self.w.write_all(&[0xcb])?;
        self.w.write_all(&value.to_bits().to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn encode_f32(&mut self, value: f32) -> Result<(), MsgPackErr> {
        self.w.write_all(&[0xca])?;
        self.w.write_all(&value.to_bits().to_be_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32;
    use std::f64;
    use std::io::Cursor;

    fn encode_f32_to_vec(v: f32) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_f32(v).unwrap();
        buf
    }

    fn encode_f64_to_vec(v: f64) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_f64(v).unwrap();
        buf
    }

    #[test]
    fn test_encode_f32_basic_values() {
        assert_eq!(encode_f32_to_vec(0.0), vec![0xca, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode_f32_to_vec(-0.0), vec![0xca, 0x80, 0x00, 0x00, 0x00]);
        assert_eq!(encode_f32_to_vec(1.0), vec![0xca, 0x3f, 0x80, 0x00, 0x00]);
        assert_eq!(encode_f32_to_vec(-1.0), vec![0xca, 0xbf, 0x80, 0x00, 0x00]);
    }

    #[test]
    fn test_encode_f64_basic_values() {
        assert_eq!(
            encode_f64_to_vec(0.0),
            vec![0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            encode_f64_to_vec(-0.0),
            vec![0xcb, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            encode_f64_to_vec(1.0),
            vec![0xcb, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            encode_f64_to_vec(-1.0),
            vec![0xcb, 0xbf, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_encode_f32_inf_nan() {
        // +∞
        assert_eq!(
            encode_f32_to_vec(f32::INFINITY),
            vec![0xca, 0x7f, 0x80, 0x00, 0x00]
        );
        // -∞
        assert_eq!(
            encode_f32_to_vec(f32::NEG_INFINITY),
            vec![0xca, 0xff, 0x80, 0x00, 0x00]
        );
        // NaN (any quiet NaN bit pattern)
        let encoded = encode_f32_to_vec(f32::NAN);
        assert_eq!(encoded[0], 0xca);
        let bits = u32::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits & 0x7f80_0000, 0x7f80_0000, "NaN exponent bits set");
        assert_ne!(bits & 0x007f_ffff, 0, "NaN mantissa nonzero");
    }

    #[test]
    fn test_encode_f64_inf_nan() {
        // +∞
        assert_eq!(
            encode_f64_to_vec(f64::INFINITY),
            vec![0xcb, 0x7f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        // -∞
        assert_eq!(
            encode_f64_to_vec(f64::NEG_INFINITY),
            vec![0xcb, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        // NaN
        let encoded = encode_f64_to_vec(f64::NAN);
        assert_eq!(encoded[0], 0xcb);
        let bits = u64::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits & 0x7ff0_0000_0000_0000, 0x7ff0_0000_0000_0000);
        assert_ne!(bits & 0x000f_ffff_ffff_ffff, 0);
    }

    #[test]
    fn test_encode_f32_subnormals() {
        // smallest positive subnormal
        let v = f32::from_bits(1);
        let encoded = encode_f32_to_vec(v);
        assert_eq!(encoded[0], 0xca);
        let bits = u32::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits, 1);
    }

    #[test]
    fn test_encode_f64_subnormals() {
        // smallest positive subnormal
        let v = f64::from_bits(1);
        let encoded = encode_f64_to_vec(v);
        assert_eq!(encoded[0], 0xcb);
        let bits = u64::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits, 1);
    }

    #[test]
    fn test_encode_f32_random_values() {
        let vals = [1.5, -3.75, 1000.125, -0.000244140625];
        for &v in &vals {
            let encoded = encode_f32_to_vec(v);
            assert_eq!(encoded[0], 0xca);
            let bits = u32::from_be_bytes(encoded[1..].try_into().unwrap());
            assert_eq!(bits, v.to_bits());
        }
    }

    #[test]
    fn test_encode_f64_random_values() {
        let vals = [1.5, -3.75, 1000.125, -0.000244140625];
        for &v in &vals {
            let encoded = encode_f64_to_vec(v);
            assert_eq!(encoded[0], 0xcb);
            let bits = u64::from_be_bytes(encoded[1..].try_into().unwrap());
            assert_eq!(bits, v.to_bits());
        }
    }

    #[test]
    fn test_encode_f32_negative_zero_distinction() {
        let pos = encode_f32_to_vec(0.0);
        let neg = encode_f32_to_vec(-0.0);
        assert_ne!(
            pos, neg,
            "positive and negative zero should differ in sign bit"
        );
    }

    #[test]
    fn test_encode_f64_negative_zero_distinction() {
        let pos = encode_f64_to_vec(0.0);
        let neg = encode_f64_to_vec(-0.0);
        assert_ne!(
            pos, neg,
            "positive and negative zero should differ in sign bit"
        );
    }

    #[test]
    fn test_encode_f32_large_exponent_boundaries() {
        let max_norm = f32::from_bits(0x7f7f_ffff);
        let encoded = encode_f32_to_vec(max_norm);
        assert_eq!(encoded[0], 0xca);
        let bits = u32::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits, 0x7f7f_ffff);

        let min_norm = f32::from_bits(0x0080_0000);
        let encoded = encode_f32_to_vec(min_norm);
        assert_eq!(encoded[0], 0xca);
        let bits = u32::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits, 0x0080_0000);
    }

    #[test]
    fn test_encode_f64_large_exponent_boundaries() {
        let max_norm = f64::from_bits(0x7fef_ffff_ffff_ffff);
        let encoded = encode_f64_to_vec(max_norm);
        assert_eq!(encoded[0], 0xcb);
        let bits = u64::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits, 0x7fef_ffff_ffff_ffff);

        let min_norm = f64::from_bits(0x0010_0000_0000_0000);
        let encoded = encode_f64_to_vec(min_norm);
        assert_eq!(encoded[0], 0xcb);
        let bits = u64::from_be_bytes(encoded[1..].try_into().unwrap());
        assert_eq!(bits, 0x0010_0000_0000_0000);
    }

    #[test]
    fn test_encode_f64_bitwise_equivalence_roundtrip() {
        // For random bit patterns
        let samples = [
            0x1234_5678_9abc_def0u64,
            0x7fe0_0000_0000_0001u64,
            0xfff8_0000_0000_0000u64,
            0x000f_ffff_ffff_ffffu64,
        ];
        for bits in samples {
            let val = f64::from_bits(bits);
            let encoded = encode_f64_to_vec(val);
            assert_eq!(encoded[0], 0xcb);
            let reencoded_bits = u64::from_be_bytes(encoded[1..].try_into().unwrap());
            assert_eq!(reencoded_bits, bits);
        }
    }

    #[test]
    fn test_encode_f32_endianness_consistency() {
        let v = 42.42_f32;
        let encoded = encode_f32_to_vec(v);
        let mut manual = vec![0xca];
        manual.extend_from_slice(&v.to_bits().to_be_bytes());
        assert_eq!(encoded, manual);
    }

    #[test]
    fn test_encode_f64_endianness_consistency() {
        let v = 42.42_f64;
        let encoded = encode_f64_to_vec(v);
        let mut manual = vec![0xcb];
        manual.extend_from_slice(&v.to_bits().to_be_bytes());
        assert_eq!(encoded, manual);
    }

    #[test]
    fn test_encode_f64_write_failure() {
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let mut writer = FailingWriter;
        let mut enc = Encoder::new(&mut writer);
        let err = enc.encode_f64(123.456).unwrap_err();
        if let MsgPackErr::Io(_) = err {
        } else {
            panic!("Expected MsgPackErr::Io, got {:?}", err);
        }
    }
}
