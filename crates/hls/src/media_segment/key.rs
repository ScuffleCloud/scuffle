use std::{fmt::Display, io};

use crate::{Tag, basic::ExtVersion};

#[derive(Debug, PartialEq, Eq)]
pub enum KeyMethod {
    Aes128,
    SampleAes,
}

impl Display for KeyMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyMethod::Aes128 => write!(f, "AES-128"),
            KeyMethod::SampleAes => write!(f, "SAMPLE-AES"),
        }
    }
}

#[derive(Debug)]
pub struct KeyAes {
    method: KeyMethod,
    uri: url::Url,
    iv: Option<u128>,
    key_format: Option<String>,
    key_format_versions: Option<String>,
}

impl Display for KeyAes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "METHOD={},URI=\"{}\"", self.method, self.uri)?;
        if let Some(iv) = self.iv {
            write!(f, ",IV=0x{iv:X}")?;
        }
        if let Some(key_format) = self.key_format.as_ref() {
            write!(f, ",KEYFORMAT=\"{key_format}\"")?;
        }
        if let Some(key_format_versions) = self.key_format_versions.as_ref() {
            write!(f, ",KEYFORMATVERSIONS=\"{key_format_versions}\"")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Key {
    None,
    Aes(KeyAes),
}

impl Tag for Key {
    const NAME: &'static str = "EXT-X-KEY";

    fn min_version(&self) -> ExtVersion {
        let mut version = ExtVersion::default();

        if let Key::Aes(aes) = self {
            if aes.iv.is_some() {
                version = ExtVersion(2);
            }
            if aes.key_format.is_some() || aes.key_format_versions.is_some() {
                version = version.max(ExtVersion(5));
            }
        }

        version
    }

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        match self {
            Key::None => write!(writer, ":METHOD=NONE"),
            Key::Aes(aes) => write!(writer, ":{aes}"),
        }
    }
}
