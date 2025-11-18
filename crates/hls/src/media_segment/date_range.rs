use std::{collections::HashMap, io};

use crate::{AttributeName, Tag};

#[derive(Debug)]
pub struct DateRange {
    pub id: String,
    pub details: DateRangeDetails,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub planned_duration: Option<chrono::Duration>,
    pub client_attributes: HashMap<AttributeName, ClientAttributeValue>,
    pub scte35_cmd: Option<u64>,
    pub scte35_out: Option<u64>,
    pub scte35_in: Option<u64>,
}

#[derive(Debug)]
pub enum DateRangeDetails {
    Normal {
        class: Option<String>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        duration: Option<chrono::Duration>,
    },
    EndOnNext {
        class: String,
    },
}

#[derive(Debug)]
pub enum ClientAttributeValue {
    HexadecimalSeq(u64),
    Float(f64),
    QuotedString(String),
}

impl ClientAttributeValue {
    fn write(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        match self {
            Self::HexadecimalSeq(v) => write!(writer, "0x{v:X}"),
            Self::Float(v) => write!(writer, "{v}"),
            Self::QuotedString(v) => {
                if v.contains(0x0a as char) || v.contains(0x0d as char) || v.contains('"') {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "quoted-string contains illegal characters",
                    ));
                }
                write!(writer, "\"{v}\"")
            }
        }
    }
}

impl Tag for DateRange {
    const NAME: &'static str = "EXT-X-DATERANGE";

    fn write_value(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        if let DateRangeDetails::Normal {
            duration: Some(duration),
            ..
        } = &self.details
            && *duration < chrono::Duration::zero()
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "illegal DURATION"));
        }

        if self.planned_duration.is_some_and(|d| d < chrono::Duration::zero()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "illegal PLANNED-DURATION"));
        }

        write!(writer, ":ID=\"{}\"", self.id)?;

        if let DateRangeDetails::Normal { class: Some(class), .. } | DateRangeDetails::EndOnNext { class } = &self.details {
            write!(writer, ",CLASS=\"{class}\"")?;
        }

        write!(writer, ",START-DATE=\"{}\"", self.start_date.format("%+"))?;

        if let DateRangeDetails::Normal {
            end_date: Some(end_date),
            ..
        } = &self.details
        {
            write!(writer, ",END-DATE=\"{}\"", end_date.format("%+"))?;
        }

        if let DateRangeDetails::Normal {
            duration: Some(duration),
            ..
        } = &self.details
        {
            write!(writer, ",DURATION={}", duration.as_seconds_f64())?;
        }

        if let Some(planned_duration) = self.planned_duration {
            write!(writer, ",PLANNED-DURATION={}", planned_duration.as_seconds_f64())?;
        }

        for (k, v) in self.client_attributes.iter() {
            write!(writer, ",{k}=")?;
            v.write(&mut writer)?;
        }

        if let Some(scte35_cmd) = self.scte35_cmd {
            write!(writer, ",SCTE35-CMD=0x{scte35_cmd:X}")?;
        }
        if let Some(scte35_out) = self.scte35_out {
            write!(writer, ",SCTE35-OUT=0x{scte35_out:X}")?;
        }
        if let Some(scte35_in) = self.scte35_in {
            write!(writer, ",SCTE35-IN=0x{scte35_in:X}")?;
        }

        if let DateRangeDetails::EndOnNext { .. } = &self.details {
            write!(writer, ",END-ON-NEXT=\"YES\"")?;
        }

        Ok(())
    }
}
