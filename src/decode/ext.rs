use crate::{
    decode::Decoder,
    error::MsgPackErr,
    value::{Extension, Value},
};
use std::io::Read;

impl<R: Read> Decoder<R> {
    pub(crate) fn decode_ext(&mut self, prefix: u8) -> Result<Value, MsgPackErr> {
        let len = match prefix {
            0xd4 => 1,
            0xd5 => 2,
            0xd6 => 4,
            0xd7 => 8,
            0xd8 => 16,
            0xc7 => self.read_u8()? as usize,
            0xc8 => self.read_u16()? as usize,
            0xc9 => self.read_u32()? as usize,
            _ => return Err(MsgPackErr::InvalidFormat(prefix)),
        };

        let ext_type = self.read_i8()?;
        let mut data = vec![0u8; len];
        let _ = self.r.read_exact(&mut data).map_err(|_| MsgPackErr::Io);

        if ext_type == -1 {
            return Ok(Value::Extension(Self::decode_timestamp(&data)?));
        }

        Ok(Value::Extension(Extension {
            type_id: ext_type,
            data,
        }))
    }

    fn decode_timestamp(data: &[u8]) -> Result<Extension, MsgPackErr> {
        match data.len() {
            4 => {
                let secs = u32::from_be_bytes(data.try_into().unwrap()) as u64;
                Ok(Extension {
                    type_id: -1,
                    data: secs.to_be_bytes().to_vec(),
                })
            }
            8 => {
                let raw = u64::from_be_bytes(data.try_into().unwrap());
                let nanos = (raw >> 34) as u32;
                let secs = raw & 0x3FFFFFFFF;
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&secs.to_be_bytes());
                bytes.extend_from_slice(&nanos.to_be_bytes());
                Ok(Extension {
                    type_id: -1,
                    data: bytes,
                })
            }
            12 => {
                let nanos = u32::from_be_bytes(data[0..4].try_into().unwrap());
                let secs = i64::from_be_bytes(data[4..12].try_into().unwrap());
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&secs.to_be_bytes());
                bytes.extend_from_slice(&nanos.to_be_bytes());
                Ok(Extension {
                    type_id: -1,
                    data: bytes,
                })
            }
            _ => Err(MsgPackErr::InvalidFormat(0xff)),
        }
    }
}
