use std::io;
use std::ops::Deref;

use base64::Engine;
use scuffle_bytes_util::BitWriter;
use scuffle_bytes_util::zero_copy::{Deserialize, Serialize, ZeroCopyReader};

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

impl Serialize for Utf8String {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(self.0.as_bytes())?;
        writer.write_all(&[0])?;
        Ok(())
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

impl Serialize for Base64String {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        let encoded = base64::prelude::BASE64_STANDARD.encode(&self.0);
        writer.write_all(encoded.as_bytes())?;
        writer.write_all(&[0])?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Utf8List(pub Vec<String>);

impl<'a> Deserialize<'a> for Utf8List {
    fn deserialize<R>(mut reader: R) -> io::Result<Self>
    where
        R: ZeroCopyReader<'a>,
    {
        let mut strings = Vec::new();

        'list: loop {
            let mut bytes = Vec::new();

            'string: loop {
                let byte = u8::deserialize(&mut reader)?;
                if byte == b' ' {
                    break 'string;
                } else if byte == 0 {
                    break 'list;
                }
                bytes.push(byte);
            }

            let string = String::from_utf8(bytes)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence"))?;

            strings.push(string);
        }

        Ok(Self(strings))
    }
}

impl Serialize for Utf8List {
    fn serialize<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: std::io::Write,
    {
        for string in &self.0 {
            writer.write_all(string.as_bytes())?;
            writer.write_all(b" ")?;
        }
        writer.write_all(&[0])?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Langauge(pub [u8; 3]);

impl<'a> Deserialize<'a> for Langauge {
    fn deserialize<R>(reader: R) -> std::io::Result<Self>
    where
        R: scuffle_bytes_util::zero_copy::ZeroCopyReader<'a>,
    {
        // 0 xxxxx xxxxx xxxxx
        let language = u16::deserialize(reader)?;
        let language = [
            ((language >> 10) & 0b11111) as u8,
            ((language >> 5) & 0b11111) as u8,
            (language & 0b11111) as u8,
        ];

        Ok(Langauge(language))
    }
}

impl Serialize for Langauge {
    fn serialize<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut bit_writer = BitWriter::new(writer);
        bit_writer.write_bit(false)?;
        bit_writer.write_bits(self.0[0] as u64, 5)?;
        bit_writer.write_bits(self.0[1] as u64, 5)?;
        bit_writer.write_bits(self.0[2] as u64, 5)?;
        Ok(())
    }
}

impl Langauge {
    pub fn code(&self) -> [char; 3] {
        [
            (self.0[0] + 0x60) as char,
            (self.0[1] + 0x60) as char,
            (self.0[2] + 0x60) as char,
        ]
    }
}
