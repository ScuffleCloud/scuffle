use std::io;

use crate::Tag;

pub enum SessionDataType {
    Value(String),
    Uri(url::Url),
}

pub struct SessionData {
    pub data_id: String,
    pub typ: SessionDataType,
    pub language: Option<String>,
}

impl Tag for SessionData {
    const NAME: &'static str = "EXT-X-SESSION-DATA";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        write!(writer, ":DATA-ID=\"{}\"", self.data_id)?;

        match &self.typ {
            SessionDataType::Value(v) => write!(writer, ",VALUE={}", v)?,
            SessionDataType::Uri(u) => write!(writer, ",URI={}", u)?,
        }

        if let Some(lang) = self.language.as_ref() {
            write!(writer, ",LANGUAGE={}", lang)?;
        }

        Ok(())
    }
}
