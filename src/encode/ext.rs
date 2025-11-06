use crate::{encode::Encoder, error::MsgPackErr, value::Extension};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_ext(&mut self, e: &Extension) -> Result<(), MsgPackErr> {
        let type_id = e.type_id;
        let data = &e.data;
        let len = data.len();

        if type_id == -1 {
            return self.encode_timestamp_payload(data);
        }

        match len {
            1 => self.w.write_all(&[0xd4])?,
            2 => self.w.write_all(&[0xd5])?,
            4 => self.w.write_all(&[0xd6])?,
            8 => self.w.write_all(&[0xd7])?,
            16 => self.w.write_all(&[0xd8])?,
            _ if u8::try_from(len).is_ok() => {
                self.w.write_all(&[0xc7])?;
                self.w.write_all(&[u8::try_from(len).unwrap()])?;
            }
            _ if u16::try_from(len).is_ok() => {
                self.w.write_all(&[0xc8])?;
                self.w
                    .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
            }
            _ => {
                self.w.write_all(&[0xc9])?;
                self.w
                    .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
            }
        }

        self.w.write_all(&[type_id as u8])?;
        self.w.write_all(data)?;
        Ok(())
    }

    fn encode_timestamp_payload(&mut self, data: &[u8]) -> Result<(), MsgPackErr> {
        match data.len() {
            4 => {
                self.w.write_all(&[0xd6, 0xff])?;
                self.w.write_all(data)?;
            }
            8 => {
                self.w.write_all(&[0xd7, 0xff])?;
                self.w.write_all(data)?;
            }
            12 => {
                self.w.write_all(&[0xc7, 12, 0xff])?;
                self.w.write_all(data)?;
            }
            _ => return Err(MsgPackErr::InvalidFormat(0xc9)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Extension;
    use std::io::{Cursor, Write};

    fn encode_ext_to_vec(ext: &Extension) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_ext(ext).unwrap();
        buf
    }

    #[test]
    fn test_encode_fixext_1() {
        let ext = Extension {
            type_id: 42,
            data: vec![0xaa],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded, vec![0xd4, 42, 0xaa]);
    }

    #[test]
    fn test_encode_fixext_2() {
        let ext = Extension {
            type_id: 1,
            data: vec![0xde, 0xad],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded, vec![0xd5, 1, 0xde, 0xad]);
    }

    #[test]
    fn test_encode_fixext_4_8_16() {
        let ext4 = Extension {
            type_id: 2,
            data: vec![0x11, 0x22, 0x33, 0x44],
        };

        assert_eq!(
            encode_ext_to_vec(&ext4),
            vec![0xd6, 2, 0x11, 0x22, 0x33, 0x44]
        );

        let ext8 = Extension {
            type_id: 3,
            data: vec![0xaa; 8],
        };

        let mut expected = vec![0xd7, 3];
        expected.extend(std::iter::repeat(0xaa).take(8));
        assert_eq!(encode_ext_to_vec(&ext8), expected);

        let ext16 = Extension {
            type_id: 4,
            data: vec![0xbb; 16],
        };

        let mut expected = vec![0xd8, 4];
        expected.extend(std::iter::repeat(0xbb).take(16));
        assert_eq!(encode_ext_to_vec(&ext16), expected);
    }

    #[test]
    fn test_encode_ext8_small_data() {
        let ext = Extension {
            type_id: 10,
            data: vec![0x99; 30],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded[0], 0xc7);
        assert_eq!(encoded[1], 30); // length
        assert_eq!(encoded[2], 10); // type_id
        assert!(encoded[3..].iter().all(|&b| b == 0x99));
    }

    #[test]
    fn test_encode_ext16_medium_data() {
        let ext = Extension {
            type_id: 11,
            data: vec![0xcc; 300],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded[0], 0xc8);
        assert_eq!(&encoded[1..3], &300u16.to_be_bytes());
        assert_eq!(encoded[3], 11);
        assert!(encoded[4..].iter().all(|&b| b == 0xcc));
    }

    #[test]
    fn test_encode_ext32_large_data() {
        let ext = Extension {
            type_id: 12,
            data: vec![0xdd; 70_000],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded[0], 0xc9);
        assert_eq!(&encoded[1..5], &70_000u32.to_be_bytes());
        assert_eq!(encoded[5], 12);
        assert!(encoded[6..].iter().all(|&b| b == 0xdd));
    }

    #[test]
    fn test_encode_timestamp32() {
        let ext = Extension {
            type_id: -1,
            data: vec![0x00, 0x00, 0x00, 0x01],
        };
        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded, vec![0xd6, 0xff, 0x00, 0x00, 0x00, 0x01]);
    }

    #[test]
    fn test_encode_timestamp64() {
        let ext = Extension {
            type_id: -1,
            data: vec![0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(
            encoded,
            vec![0xd7, 0xff, 0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn test_encode_timestamp96() {
        let data = vec![
            0x00, 0x00, 0x00, 0x01, // nsec
            0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef, // sec
        ];

        let ext = Extension { type_id: -1, data };
        let encoded = encode_ext_to_vec(&ext);
        let expected_prefix = vec![0xc7, 12, 0xff];
        assert_eq!(&encoded[..3], &expected_prefix[..]);
        assert_eq!(encoded.len(), 3 + 12);
    }

    #[test]
    fn test_encode_timestamp_invalid_size() {
        let ext = Extension {
            type_id: -1,
            data: vec![1, 2, 3], // invalid len
        };

        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        let err = enc.encode_ext(&ext).unwrap_err();
        if let MsgPackErr::InvalidFormat(0xc9) = err {
        } else {
            panic!("Expected InvalidFormat(0xc9), got {:?}", err);
        }
    }

    #[test]
    fn test_encode_ext_with_min_max_type_ids() {
        let ext_min = Extension {
            type_id: -128,
            data: vec![0xaa],
        };

        let ext_max = Extension {
            type_id: 127,
            data: vec![0xaa],
        };

        let encoded_min = encode_ext_to_vec(&ext_min);
        let encoded_max = encode_ext_to_vec(&ext_max);
        assert_eq!(encoded_min[1], 0x80); // -128 as u8
        assert_eq!(encoded_max[1], 127);
    }

    #[test]
    fn test_encode_ext_zero_length_data() {
        // This should choose ext8 with len = 0
        let ext = Extension {
            type_id: 5,
            data: vec![],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded, vec![0xc7, 0x00, 5]);
    }

    #[test]
    fn test_encode_ext_large_data_boundary() {
        let len = 65_535;
        let ext = Extension {
            type_id: 6,
            data: vec![0x11; len],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded[0], 0xc8);
        assert_eq!(&encoded[1..3], &(len as u16).to_be_bytes());
        assert_eq!(encoded[3], 6);
        assert_eq!(encoded.len(), 4 + len);
    }

    #[test]
    fn test_encode_ext_huge_data_map32() {
        let len = 70_000;
        let ext = Extension {
            type_id: 7,
            data: vec![0x22; len],
        };

        let encoded = encode_ext_to_vec(&ext);
        assert_eq!(encoded[0], 0xc9);
        assert_eq!(&encoded[1..5], &(len as u32).to_be_bytes());
        assert_eq!(encoded[5], 7);
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
                    "simulated failure",
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
    fn test_encode_ext_write_failure() {
        let mut writer = FailingWriter {
            fail_after: 3,
            written: 0,
        };

        let mut enc = Encoder::new(&mut writer);
        let ext = Extension {
            type_id: 1,
            data: vec![0xaa, 0xbb, 0xcc, 0xdd],
        };
        let err = enc.encode_ext(&ext).unwrap_err();
        if let MsgPackErr::Io(_) = err {
        } else {
            panic!("Expected MsgPackErr::Io, got {:?}", err);
        }
    }
}
