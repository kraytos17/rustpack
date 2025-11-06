use crate::{encode::Encoder, error::MsgPackErr};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_str(&mut self, s: &str) -> Result<(), MsgPackErr> {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if len <= 31 {
            self.w.write_all(&[(0xa0 | u8::try_from(len).unwrap())])?;
        } else if u8::try_from(len).is_ok() {
            self.w.write_all(&[0xd9])?;
            self.w.write_all(&[u8::try_from(len).unwrap()])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xda])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xdb])?;
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

    fn encode_str_to_vec(s: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_str(s).unwrap();
        buf
    }

    #[test]
    fn test_encode_empty_string() {
        let encoded = encode_str_to_vec("");
        assert_eq!(encoded, vec![0xa0]); // fixstr, len=0
    }

    #[test]
    fn test_encode_small_fixstr() {
        let encoded = encode_str_to_vec("abc");
        // 0xa0 + 3 = 0xa3
        assert_eq!(encoded, vec![0xa3, b'a', b'b', b'c']);
    }

    #[test]
    fn test_encode_fixstr_boundary_31() {
        let s = "x".repeat(31);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xbf); // 0xa0 | 31
        assert_eq!(encoded.len(), 1 + 31);
    }

    #[test]
    fn test_encode_str8_transition_32() {
        let s = "y".repeat(32);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xd9); // str8
        assert_eq!(encoded[1], 32);
        assert_eq!(encoded.len(), 2 + 32);
    }

    #[test]
    fn test_encode_str8_max_255() {
        let s = "z".repeat(255);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xd9);
        assert_eq!(encoded[1], 255);
        assert_eq!(encoded.len(), 2 + 255);
    }

    #[test]
    fn test_encode_str16_transition_256() {
        let s = "a".repeat(256);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xda);
        assert_eq!(&encoded[1..3], &256u16.to_be_bytes());
        assert_eq!(encoded.len(), 3 + 256);
    }

    #[test]
    fn test_encode_str16_max_65535() {
        let s = "b".repeat(65535);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xda);
        assert_eq!(&encoded[1..3], &65535u16.to_be_bytes());
        assert_eq!(encoded.len(), 3 + 65535);
    }

    #[test]
    fn test_encode_str32_transition_65536() {
        let s = "c".repeat(65536);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xdb);
        assert_eq!(&encoded[1..5], &65536u32.to_be_bytes());
        assert_eq!(encoded.len(), 5 + 65536);
    }

    #[test]
    fn test_encode_str32_large() {
        let len = 100_000;
        let s = "d".repeat(len);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xdb);
        assert_eq!(&encoded[1..5], &(len as u32).to_be_bytes());
        assert_eq!(encoded.len(), 5 + len);
    }

    #[test]
    fn test_encode_unicode_emojis() {
        let s = "😀😁😂🤣😃😄😅";
        let encoded = encode_str_to_vec(s);
        assert_eq!(encoded[0], 0xa0 | (s.len() as u8));
        assert_eq!(encoded.len(), 1 + s.len());
        let decoded = std::str::from_utf8(&encoded[1..]).unwrap();
        assert_eq!(decoded, s);
    }

    #[test]
    fn test_encode_utf8_multibyte_char_boundary() {
        let s = "𐍈".repeat(16);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xd9);
        assert_eq!(encoded[1], 64);
        assert_eq!(encoded.len(), 2 + 64);
    }

    #[test]
    fn test_encode_str_zero_length_but_with_nonempty_capacity() {
        let mut s = String::new();
        s.reserve(1000);
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded, vec![0xa0]);
    }

    #[test]
    fn test_encode_str_high_ascii_boundary() {
        let s: String = (0x20u8..0x80u8).map(|b| b as char).collect();
        let encoded = encode_str_to_vec(&s);
        assert!(encoded[0] == 0xd9 || encoded[0] == 0xda);
        assert_eq!(&encoded[2..], s.as_bytes());
    }

    #[test]
    fn test_encode_str_huge_memory_but_small_data() {
        let mut s = String::with_capacity(10_000_000);
        s.push_str("hello");
        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded, vec![0xa5, b'h', b'e', b'l', b'l', b'o']);
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
    fn test_encode_str_write_failure() {
        let mut writer = FailingWriter {
            fail_after: 2,
            written: 0,
        };

        let mut enc = Encoder::new(&mut writer);
        let err = enc.encode_str("abcdef").unwrap_err();
        if let MsgPackErr::Io(_) = err {
        } else {
            panic!("Expected MsgPackErr::Io, got {:?}", err);
        }
    }

    #[test]
    fn test_encode_str_random_large_content() {
        let s: String = (0..10_000)
            .map(|i| ((i % 26) as u8 + b'a') as char)
            .collect();

        let encoded = encode_str_to_vec(&s);
        assert_eq!(encoded[0], 0xda);
        assert_eq!(&encoded[1..3], &(10_000u16.to_be_bytes()));
        assert_eq!(&encoded[3..], s.as_bytes());
    }
}
