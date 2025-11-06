use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_bin(&mut self, bytes: &[u8]) -> Result<(), MsgPackErr> {
        let len = bytes.len();
        if u8::try_from(len).is_ok() {
            self.w.write_all(&[0xc4])?;
            self.w.write_all(&[u8::try_from(len).unwrap()])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xc5])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xc6])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        self.w.write_all(bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn encode_to_vec(bytes: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        Encoder::new(Cursor::new(&mut buf))
            .encode_bin(bytes)
            .unwrap();
        buf
    }

    #[test]
    fn test_encode_bin_empty() {
        // Bin8 with length 0
        let encoded = encode_to_vec(&[]);
        assert_eq!(encoded, vec![0xc4, 0x00]);
    }

    #[test]
    fn test_encode_bin8_min() {
        let data = vec![0x11];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded, vec![0xc4, 0x01, 0x11]);
    }

    #[test]
    fn test_encode_bin8_max() {
        let data = vec![0xAA; 255];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc4);
        assert_eq!(encoded[1], 255);
        assert_eq!(encoded.len(), 2 + 255);
        assert!(encoded[2..].iter().all(|&b| b == 0xAA));
    }

    #[test]
    fn test_encode_bin16_transition() {
        let data = vec![0xBB; 256];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc5);
        assert_eq!(&encoded[1..3], &[0x01, 0x00]);
        assert_eq!(encoded.len(), 3 + 256);
        assert!(encoded[3..].iter().all(|&b| b == 0xBB));
    }

    #[test]
    fn test_encode_bin16_max() {
        let data = vec![0xCC; 65_535];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc5);
        assert_eq!(&encoded[1..3], &[0xFF, 0xFF]);
        assert_eq!(encoded.len(), 3 + 65_535);
        assert!(encoded[3..].iter().all(|&b| b == 0xCC));
    }

    #[test]
    fn test_encode_bin32_transition() {
        let data = vec![0xDD; 65_536];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc6);
        assert_eq!(&encoded[1..5], &[0x00, 0x01, 0x00, 0x00]);
        assert_eq!(encoded.len(), 5 + 65_536);
        assert!(encoded[5..].iter().all(|&b| b == 0xDD));
    }

    #[test]
    fn test_encode_bin32_large_random_data() {
        let data = (0..10_000).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc5);
        let len_bytes = &encoded[1..3];
        let len = u16::from_be_bytes(len_bytes.try_into().unwrap());
        assert_eq!(len, data.len() as u16);
        assert_eq!(&encoded[3..], &data[..]);
    }

    #[test]
    fn test_encode_bin_single_byte_edge_cases() {
        for val in [0x00, 0x7F, 0x80, 0xFF] {
            let data = vec![val];
            let encoded = encode_to_vec(&data);
            assert_eq!(encoded, vec![0xc4, 0x01, val]);
        }
    }

    #[test]
    fn test_encode_bin_with_pattern_data() {
        let data = (0..32).map(|i| (i * 7 % 256) as u8).collect::<Vec<_>>();
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc4);
        assert_eq!(encoded[1], 32);
        assert_eq!(&encoded[2..], &data[..]);
    }

    #[test]
    fn test_encode_bin32_large_limit_check() {
        let len: usize = 1_000_000;
        let data = vec![0x42; len];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc6);
        let reported_len =
            u32::from_be_bytes(encoded[1..5].try_into().expect("Invalid length prefix"));
        assert_eq!(reported_len, len as u32);
        assert_eq!(encoded.len(), 5 + len);
    }

    #[test]
    fn test_encode_bin_varied_lengths() {
        fn encoded_prefix_len(prefix: u8) -> usize {
            match prefix {
                0xc4 => 2,
                0xc5 => 3,
                0xc6 => 5,
                _ => panic!("Invalid prefix"),
            }
        }

        let test_cases = [0, 1, 127, 128, 255, 256, 257, 65_535, 65_536];
        for len in test_cases {
            let data = vec![0xAB; len];
            let encoded = encode_to_vec(&data);
            if len <= 255 {
                assert_eq!(encoded[0], 0xc4);
            } else if len <= 65_535 {
                assert_eq!(encoded[0], 0xc5);
            } else {
                assert_eq!(encoded[0], 0xc6);
            }
            assert_eq!(encoded.len(), encoded_prefix_len(encoded[0]) + len);
        }
    }

    #[test]
    fn test_encode_bin_does_not_mutate_input() {
        let data = vec![1, 2, 3, 4];
        let original = data.clone();
        let _ = encode_to_vec(&data);
        assert_eq!(data, original);
    }

    #[test]
    fn test_encode_bin_very_large_boundary_bytes() {
        let data = vec![0xFF; 1_000_000];
        let encoded = encode_to_vec(&data);
        assert_eq!(encoded[0], 0xc6);
        assert!(encoded[5..].iter().all(|&b| b == 0xFF));
    }

    #[test]
    #[should_panic]
    fn test_encode_bin_overflow_length() {
        // simulate >u32::MAX size — expect panic or failure
        let len = (u32::MAX as usize).checked_add(1).unwrap();
        let _ = encode_to_vec(&vec![0; len]);
    }

    struct FailingWriter {
        fail_after: usize,
        written: usize,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.written + buf.len() > self.fail_after {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "simulated write failure",
                ));
            }
            self.written += buf.len();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_encode_bin_write_failure() {
        let mut writer = FailingWriter {
            fail_after: 3,
            written: 0,
        };

        let mut enc = Encoder::new(&mut writer);
        let err = enc.encode_bin(&vec![0u8; 10]).unwrap_err();
        assert!(writer.written <= 3);
        if let MsgPackErr::Io(_) = err {
        } else {
            panic!("Expected MsgPackErr::Io, got {:?}", err);
        }
    }

    #[test]
    fn test_encode_bin_static_and_subslice() {
        static DATA: [u8; 3] = [0, 1, 2];
        let encoded = encode_to_vec(&DATA[1..]);
        assert_eq!(encoded, vec![0xc4, 0x02, 0x01, 0x02]);
    }
}
