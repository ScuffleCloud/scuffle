use std::collections::HashMap;
use std::io;

use bytes::Bytes;
use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Object, Amf0Value};
use scuffle_bytes_util::BytesCursorExt;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct OnMetaData {
    pub audiocodecid: Option<f64>,
    pub audiodatarate: Option<f64>,
    pub audiodelay: Option<f64>,
    pub audiosamplerate: Option<f64>,
    pub audiosamplesize: Option<f64>,
    pub can_seek_to_end: Option<bool>,
    pub creationdate: Option<String>,
    pub duration: Option<f64>,
    pub filesize: Option<f64>,
    pub framerate: Option<f64>,
    pub height: Option<f64>,
    pub stereo: Option<bool>,
    pub videocodecid: Option<f64>,
    pub videodatarate: Option<f64>,
    pub width: Option<f64>,
    pub other: HashMap<String, Amf0Value<'static>>,
}

// Be warned: Insanely ugly code ahead
// We should maybe implement serde support in the amf0 crate

impl TryFrom<Amf0Object<'_>> for OnMetaData {
    type Error = Error;

    fn try_from(value: Amf0Object) -> Result<Self, Self::Error> {
        let mut other = HashMap::new();

        let mut audiocodecid = None;
        let mut audiodatarate = None;
        let mut audiodelay = None;
        let mut audiosamplerate = None;
        let mut audiosamplesize = None;
        let mut can_seek_to_end = None;
        let mut creationdate = None;
        let mut duration = None;
        let mut filesize = None;
        let mut framerate = None;
        let mut height = None;
        let mut stereo = None;
        let mut videocodecid = None;
        let mut videodatarate = None;
        let mut width = None;

        for (key, value) in value.iter() {
            match key.as_ref() {
                "audiocodecid" => audiocodecid = Some(value.as_number()?),
                "audiodatarate" => audiodatarate = Some(value.as_number()?),
                "audiodelay" => audiodelay = Some(value.as_number()?),
                "audiosamplerate" => audiosamplerate = Some(value.as_number()?),
                "audiosamplesize" => audiosamplesize = Some(value.as_number()?),
                "canSeekToEnd" => can_seek_to_end = Some(value.as_boolean()?),
                "creationdate" => creationdate = Some(value.as_string()?.to_string()),
                "duration" => duration = Some(value.as_number()?),
                "filesize" => filesize = Some(value.as_number()?),
                "framerate" => framerate = Some(value.as_number()?),
                "height" => height = Some(value.as_number()?),
                "stereo" => stereo = Some(value.as_boolean()?),
                "videocodecid" => videocodecid = Some(value.as_number()?),
                "videodatarate" => videodatarate = Some(value.as_number()?),
                "width" => width = Some(value.as_number()?),
                _ => {
                    other.insert(key.to_string(), value.to_owned());
                }
            }
        }

        Ok(Self {
            audiocodecid,
            audiodatarate,
            audiodelay,
            audiosamplerate,
            audiosamplesize,
            can_seek_to_end,
            creationdate,
            duration,
            filesize,
            framerate,
            height,
            stereo,
            videocodecid,
            videodatarate,
            width,
            other,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnXmpData {
    live_xml: Option<String>,
    other: HashMap<String, Amf0Value<'static>>,
}

impl TryFrom<Amf0Object<'_>> for OnXmpData {
    type Error = Error;

    fn try_from(value: Amf0Object<'_>) -> Result<Self, Self::Error> {
        let mut other = HashMap::new();

        let mut live_xml = None;

        for (key, value) in value.iter() {
            if key == "liveXML" {
                live_xml = Some(value.as_string()?.to_string());
            } else {
                other.insert(key.to_string(), value.to_owned());
            }
        }

        Ok(Self { live_xml, other })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptData {
    OnMetaData(OnMetaData),
    OnXmpData(OnXmpData),
    Other {
        /// The name of the script data
        name: String,
        /// The data of the script data
        data: Vec<Amf0Value<'static>>,
    },
}

impl ScriptData {
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, Error> {
        let buf = reader.extract_remaining();
        let mut amf0_reader = Amf0Decoder::new(&buf);

        let Amf0Value::String(name) = amf0_reader.decode_with_type(Amf0Marker::String)? else {
            unreachable!()
        };

        match name.as_ref() {
            // We might also want to handle "@setDataFrame" the same way as onMetaData.
            // I'm not sure right now if that is the intended behavior though.
            "onMetaData" => {
                let value = amf0_reader.decode()?;
                println!("{:?}", value);

                let Amf0Value::Object(data) = value else { unreachable!() };
                let data = OnMetaData::try_from(data)?;

                Ok(Self::OnMetaData(data))
            }
            "onXMPData" => {
                let value = amf0_reader.decode()?;
                println!("{:?}", value);

                let Amf0Value::Object(data) = value else { unreachable!() };
                let data = OnXmpData::try_from(data)?;

                Ok(Self::OnXmpData(data))
            }
            _ => {
                tracing::warn!(name = %name, "unknown script data");

                let data = amf0_reader.decode_all()?;

                Ok(Self::Other {
                    name: name.into_owned(),
                    data: data.into_iter().map(|v| v.to_owned()).collect(),
                })
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_script_data() {
        let width = 1280.0f64.to_be_bytes();
        let data = [
            Amf0Marker::String as u8,
            0, // Length (10 bytes)
            10,
            b'o', // "onMetaData"
            b'n',
            b'M',
            b'e',
            b't',
            b'a',
            b'D',
            b'a',
            b't',
            b'a',
            Amf0Marker::Object as u8,
            0, // Length (5 bytes)
            5,
            b'w', // "width"
            b'i',
            b'd',
            b't',
            b'h',
            Amf0Marker::Number as u8,
            width[0],
            width[1],
            width[2],
            width[3],
            width[4],
            width[5],
            width[6],
            width[7],
            0,
            0,
            Amf0Marker::ObjectEnd as u8,
        ];

        let mut reader = io::Cursor::new(Bytes::from_owner(data));

        let script_data = ScriptData::demux(&mut reader).unwrap();

        let ScriptData::OnMetaData(metadata) = script_data else {
            panic!("expected onMetaData");
        };

        assert_eq!(metadata.audiocodecid, None);
        assert_eq!(metadata.audiodatarate, None);
        assert_eq!(metadata.audiodelay, None);
        assert_eq!(metadata.audiosamplerate, None);
        assert_eq!(metadata.audiosamplesize, None);
        assert_eq!(metadata.can_seek_to_end, None);
        assert_eq!(metadata.creationdate, None);
        assert_eq!(metadata.duration, None);
        assert_eq!(metadata.filesize, None);
        assert_eq!(metadata.framerate, None);
        assert_eq!(metadata.height, None);
        assert_eq!(metadata.stereo, None);
        assert_eq!(metadata.videocodecid, None);
        assert_eq!(metadata.videodatarate, None);
        assert_eq!(metadata.width, Some(1280.0));
    }
}
