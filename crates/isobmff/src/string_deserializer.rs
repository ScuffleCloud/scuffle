use std::io;
use std::ops::Deref;

use base64::Engine;
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
            let byte = u8::deserialize(&mut reader)?;
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

#[derive(Debug)]
pub struct Base64String(pub Vec<u8>);

impl Deref for Base64String {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Deserialize<'a> for Base64String {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let mut bytes = Vec::new();

        loop {
            let byte = u8::deserialize(&mut reader)?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }

        let data = base64::prelude::BASE64_STANDARD
            .decode(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Base64 decode error: {e}")))?;
        Ok(Base64String(data))
    }
}
