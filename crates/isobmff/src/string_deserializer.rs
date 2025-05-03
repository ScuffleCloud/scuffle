use std::io;
use std::ops::Deref;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::zero_copy::{Deserialize, ZeroCopyReader};

#[derive(Debug)]
pub struct Utf8String(pub String);

impl Deref for Utf8String {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Deserialize<'a> for Utf8String {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let mut bytes = Vec::new();

        loop {
            let byte = reader.as_std().read_u8()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }

        let string =
            String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;
        Ok(Utf8String(string))
    }
}
