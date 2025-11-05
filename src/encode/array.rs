use crate::{encode::Encoder, error::MsgPackErr, value::Value};
use std::io::Write;

impl<W: Write> Encoder<W> {
    pub(crate) fn encode_arr(&mut self, arr: &[Value]) -> Result<(), MsgPackErr> {
        let len = arr.len();
        if len <= 15 {
            self.w.write_all(&[(0x90 | u8::try_from(len).unwrap())])?;
        } else if u16::try_from(len).is_ok() {
            self.w.write_all(&[0xdc])?;
            self.w
                .write_all(&u16::try_from(len).unwrap().to_be_bytes())?;
        } else {
            self.w.write_all(&[0xdd])?;
            self.w
                .write_all(&u32::try_from(len).unwrap().to_be_bytes())?;
        }

        for v in arr {
            let _ = self.encode(v);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        to_vec,
        value::{Integer, Value},
    };

    fn encode_to_vec(val: &Value) -> Vec<u8> {
        to_vec(val).unwrap()
    }

    #[test]
    fn test_encode_empty_array() {
        let arr = Value::Array(vec![]);
        assert_eq!(encode_to_vec(&arr), vec![0x90]);
    }

    #[test]
    fn test_encode_fixarray_basic() {
        let arr = Value::Array(vec![
            Value::Boolean(true),
            Value::Integer(Integer::I64(1)),
            Value::Nil,
        ]);
        assert_eq!(encode_to_vec(&arr), vec![0x93, 0xc3, 0x01, 0xc0]);
    }

    #[test]
    fn test_encode_array16_transition() {
        let arr = Value::Array(vec![Value::Nil; 16]);
        let mut expected = vec![0xdc, 0x00, 0x10];
        expected.extend(std::iter::repeat(0xc0).take(16));
        assert_eq!(encode_to_vec(&arr), expected);
    }

    #[test]
    fn test_encode_array32_transition() {
        let arr = Value::Array(vec![Value::Nil; 65536]);
        let expected_prefix = vec![0xdd, 0x00, 0x01, 0x00, 0x00];
        assert_eq!(&encode_to_vec(&arr)[..5], &expected_prefix[..]);
    }

    #[test]
    fn test_encode_deeply_nested_arrays() {
        // [[[...[[Nil]]...]]], depth 6
        let mut val = Value::Nil;
        for _ in 0..6 {
            val = Value::Array(vec![val]);
        }
        let encoded = encode_to_vec(&val);
        // expected prefix sequence: 0x91 repeated 6 times, then 0xc0
        let mut expected = vec![0x91; 6];
        expected.push(0xc0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_nested_empty_arrays() {
        // [[], [], []]
        let arr = Value::Array(vec![
            Value::Array(vec![]),
            Value::Array(vec![]),
            Value::Array(vec![]),
        ]);
        assert_eq!(encode_to_vec(&arr), vec![0x93, 0x90, 0x90, 0x90]);
    }

    #[test]
    fn test_encode_mixed_type_array_complex() {
        let arr = Value::Array(vec![
            Value::Nil,
            Value::String("abc".into()),
            Value::Integer(Integer::I64(-42)),
            Value::Binary(vec![1, 2, 3]),
            Value::Array(vec![
                Value::Boolean(false),
                Value::Integer(Integer::U64(999999)),
            ]),
        ]);

        let encoded = encode_to_vec(&arr);

        // Just check prefix correctness and structural markers
        assert_eq!(encoded[0], 0x95); // fixarray(5)
        assert_eq!(encoded[1], 0xc0); // nil
        assert!(encoded.contains(&0xa3)); // str(3)
        assert!(encoded.contains(&0xc4)); // bin(3)
        assert!(encoded.contains(&0x92)); // inner fixarray(2)
    }

    #[test]
    fn test_encode_array_with_large_numbers() {
        let arr = Value::Array(vec![
            Value::Integer(Integer::I64(i64::MIN)),
            Value::Integer(Integer::I64(i64::MAX)),
            Value::Integer(Integer::U64(u64::MAX)),
        ]);

        let encoded = encode_to_vec(&arr);

        assert_eq!(encoded[0], 0x93); // 3 elements
        assert_eq!(encoded[1], 0xd3); // int64 prefix for i64::MIN
        assert_eq!(encoded[10], 0xd3); // int64 prefix for i64::MAX
        assert_eq!(encoded[19], 0xcf); // uint64 prefix for u64::MAX
    }

    #[test]
    fn test_encode_array_with_empty_string_and_bin() {
        let arr = Value::Array(vec![
            Value::String(String::new()),
            Value::Binary(Vec::new()),
        ]);
        let encoded = encode_to_vec(&arr);
        // fixarray(2)
        // empty str = 0xa0, empty bin = 0xc4 0x00
        assert_eq!(encoded, vec![0x92, 0xa0, 0xc4, 0x00]);
    }

    #[test]
    fn test_encode_array_with_nested_map() {
        // [ { "a": 1 }, { "b": 2 } ]
        let arr = Value::Array(vec![
            Value::Map(vec![(
                Value::String("a".into()),
                Value::Integer(Integer::I64(1)),
            )]),
            Value::Map(vec![(
                Value::String("b".into()),
                Value::Integer(Integer::I64(2)),
            )]),
        ]);
        let encoded = encode_to_vec(&arr);
        // Outer = 2 elements -> 0x92
        // Each map = fixmap(1) -> 0x81
        assert_eq!(encoded[0], 0x92);
        assert!(encoded.iter().filter(|&&b| b == 0x81).count() == 2);
    }

    #[test]
    fn test_encode_array_with_various_containers() {
        // [[], {}, [1,2,3], {"x": nil}]
        let arr = Value::Array(vec![
            Value::Array(vec![]),
            Value::Map(vec![]),
            Value::Array(vec![
                Value::Integer(Integer::I64(1)),
                Value::Integer(Integer::I64(2)),
                Value::Integer(Integer::I64(3)),
            ]),
            Value::Map(vec![(Value::String("x".into()), Value::Nil)]),
        ]);
        let encoded = encode_to_vec(&arr);
        // [0x94, 0x90, 0x80, 0x93, 0x01, 0x02, 0x03, 0x81, 0xa1, b'x', 0xc0]
        assert_eq!(encoded[0], 0x94);
        assert!(encoded.ends_with(&[0xc0]));
    }

    #[test]
    fn test_encode_very_large_nested_structure() {
        // array of 1024 empty arrays
        let arr = Value::Array(vec![Value::Array(vec![]); 1024]);
        let encoded = encode_to_vec(&arr);
        // 0xdc 0x04 0x00 prefix = array16 length 1024
        assert_eq!(&encoded[..3], &[0xdc, 0x04, 0x00]);
        // All nested arrays should be 0x90 (empty array marker)
        assert!(encoded.iter().skip(3).all(|&b| b == 0x90));
    }
}
