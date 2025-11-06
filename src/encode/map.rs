use crate::{encode::Encoder, error::MsgPackErr, value::Value};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_map(&mut self, map: &[(Value, Value)]) -> Result<(), MsgPackErr> {
        let len = map.len();
        if len <= 15 {
            self.w.write_all(&[(0x80 | u8::try_from(len).unwrap())])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xde])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xdf])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        for (k, v) in map {
            self.encode(k)?;
            self.encode(v)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Integer, Value};
    use std::io::{Cursor, Write};

    fn encode_map_to_vec(map: &[(Value, Value)]) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = Encoder::new(Cursor::new(&mut buf));
        enc.encode_map(map).unwrap();
        buf
    }

    #[test]
    fn test_encode_empty_map() {
        let encoded = encode_map_to_vec(&[]);
        assert_eq!(encoded, vec![0x80]); // fixmap(0)
    }

    #[test]
    fn test_encode_fixmap_single_entry() {
        let map = &[(Value::String("a".into()), Value::Integer(Integer::I64(1)))];
        let encoded = encode_map_to_vec(map);
        assert_eq!(encoded, vec![0x81, 0xa1, b'a', 0x01]); // fixmap(1)
    }

    #[test]
    fn test_encode_fixmap_multiple_entries() {
        let map = &[
            (Value::String("x".into()), Value::Boolean(true)),
            (Value::String("y".into()), Value::Nil),
            (Value::String("z".into()), Value::Integer(Integer::I64(42))),
        ];

        let encoded = encode_map_to_vec(map);
        // prefix = fixmap(3) = 0x83
        assert_eq!(encoded[0], 0x83);
        assert!(encoded.contains(&b'x'));
        assert!(encoded.contains(&b'y'));
        assert!(encoded.contains(&b'z'));
    }

    #[test]
    fn test_encode_map16_transition() {
        let map = (0..16)
            .map(|i| (Value::Integer(Integer::U64(i)), Value::Boolean(i % 2 == 0)))
            .collect::<Vec<_>>();

        let encoded = encode_map_to_vec(&map);
        assert_eq!(encoded[0], 0xde); // map16
        assert_eq!(&encoded[1..3], &[0x00, 0x10]);
    }

    #[test]
    fn test_encode_map16_max_boundary() {
        let map = (0..65_535)
            .map(|i| (Value::Integer(Integer::U64(i as u64)), Value::Nil))
            .collect::<Vec<_>>();

        let encoded = encode_map_to_vec(&map);
        assert_eq!(encoded[0], 0xde);
        assert_eq!(&encoded[1..3], &[0xff, 0xff]);
        assert!(encoded.len() > 10);
    }

    #[test]
    fn test_encode_map32_transition() {
        let map = (0..65_536)
            .map(|i| {
                (
                    Value::Integer(Integer::U64(i as u64)),
                    Value::Integer(Integer::I64(1)),
                )
            })
            .collect::<Vec<_>>();

        let encoded = encode_map_to_vec(&map);
        assert_eq!(encoded[0], 0xdf);
        assert_eq!(&encoded[1..5], &[0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_encode_nested_maps() {
        // { "outer": { "inner": 42 } }
        let inner = Value::Map(vec![(
            Value::String("inner".into()),
            Value::Integer(Integer::I64(42)),
        )]);

        let outer = [(Value::String("outer".into()), inner)];
        let encoded = encode_map_to_vec(&outer);
        assert_eq!(encoded[0], 0x81); // fixmap(1)
        assert!(encoded.contains(&b'o'));
        assert!(encoded.contains(&b'i'));
    }

    #[test]
    fn test_encode_map_with_mixed_types() {
        let map = &[
            (Value::Nil, Value::String("null".into())),
            (Value::Boolean(true), Value::Integer(Integer::I64(123))),
            (
                Value::Integer(Integer::U64(5)),
                Value::Binary(vec![1, 2, 3]),
            ),
        ];

        let encoded = encode_map_to_vec(map);
        assert_eq!(encoded[0], 0x83); // fixmap(3)
        assert!(encoded.contains(&0xc2) || encoded.contains(&0xc3)); // bool
        assert!(encoded.contains(&0xc4)); // bin marker
    }

    #[test]
    fn test_encode_map_with_large_numbers() {
        let map = &[
            (
                Value::Integer(Integer::I64(i64::MAX)),
                Value::Integer(Integer::I64(i64::MIN)),
            ),
            (Value::Integer(Integer::U64(u64::MAX)), Value::Nil),
        ];

        let encoded = encode_map_to_vec(map);
        assert_eq!(encoded[0], 0x82); // fixmap(2)
        assert!(encoded.contains(&0xd3)); // int64 prefix
        assert!(encoded.contains(&0xcf)); // uint64 prefix
    }

    #[test]
    fn test_encode_map_with_string_keys_and_array_values() {
        let map = &[(
            Value::String("nums".into()),
            Value::Array(vec![
                Value::Integer(Integer::I64(1)),
                Value::Integer(Integer::I64(2)),
                Value::Integer(Integer::I64(3)),
            ]),
        )];

        let encoded = encode_map_to_vec(map);
        assert_eq!(encoded[0], 0x81);
        assert!(encoded.contains(&0x93)); // fixarray(3)
    }

    #[test]
    fn test_encode_map_with_empty_nested_structures() {
        let map = &[
            (Value::String("empty_map".into()), Value::Map(vec![])),
            (Value::String("empty_array".into()), Value::Array(vec![])),
        ];

        let encoded = encode_map_to_vec(map);
        assert_eq!(encoded[0], 0x82);
        assert!(encoded.contains(&0x80)); // empty map marker
        assert!(encoded.contains(&0x90)); // empty array marker
    }

    #[test]
    fn test_encode_map_large_and_complex() {
        let mut map = Vec::new();
        for i in 0..50 {
            let key = Value::String(format!("key{i}"));
            let val = Value::Array(vec![
                Value::Integer(Integer::I64(i)),
                Value::String(format!("val{i}")),
            ]);

            map.push((key, val));
        }

        let encoded = encode_map_to_vec(&map);
        assert_eq!(encoded[0], 0xde);
        let len_bytes = u16::from_be_bytes(encoded[1..3].try_into().unwrap());
        assert_eq!(len_bytes, 50);
    }

    #[test]
    fn test_encode_map_does_not_mutate_input() {
        let map = vec![(
            Value::String("key".into()),
            Value::Integer(Integer::I64(10)),
        )];

        let original = map.clone();
        let _ = encode_map_to_vec(&map);
        assert_eq!(map, original);
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
    fn test_encode_map_write_failure() {
        let mut writer = FailingWriter {
            fail_after: 2,
            written: 0,
        };

        let mut enc = Encoder::new(&mut writer);
        let map = &[(Value::String("a".into()), Value::Integer(Integer::I64(1)))];
        let err = enc.encode_map(map).unwrap_err();
        if let MsgPackErr::Io(_) = err {
        } else {
            panic!("Expected MsgPackErr::Io, got {:?}", err);
        }
    }

    #[test]
    fn test_encode_map_huge_boundary_map32() {
        let len = 70_000;
        let map = (0..len)
            .map(|i| (Value::Integer(Integer::U64(i as u64)), Value::Nil))
            .collect::<Vec<_>>();
        let encoded = encode_map_to_vec(&map);
        assert_eq!(encoded[0], 0xdf); // map32 prefix
        let len_bytes = u32::from_be_bytes(encoded[1..5].try_into().unwrap());
        assert_eq!(len_bytes, len as u32);
    }
}
