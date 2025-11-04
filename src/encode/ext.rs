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

        self.w.write_all(&[u8::try_from(type_id).unwrap()])?;
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
