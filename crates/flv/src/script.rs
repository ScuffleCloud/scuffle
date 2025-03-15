use std::collections::HashMap;
use std::io;

use bytes::Bytes;
use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Object, Amf0Value};
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::header::{AudioFourCc, SoundFormat};
use crate::error::Error;
use crate::video::header::enhanced::VideoFourCc;
use crate::video::header::legacy::VideoCodecId;

#[derive(Debug, Clone, PartialEq)]
pub enum OnMetaDataAudioCodecId {
    Legacy(SoundFormat),
    Enhanced(AudioFourCc),
}

impl OnMetaDataAudioCodecId {
    fn from_amf0(value: &Amf0Value<'_>) -> Result<Self, Error> {
        let n = value.as_number()? as u32;

        // Since SoundFormat is a u8, we can be sure that the number represents an AudioFourCc if it is greater
        // than u8::MAX.
        // Additionally, since the smallest possible AudioFourCc (4 spaces) is greater than u8::MAX,
        // we can be sure that the number cannot represent an AudioFourCc if it is smaller than u8::MAX.
        if n > u8::MAX as u32 {
            Ok(Self::Enhanced(AudioFourCc::from(n.to_be_bytes())))
        } else {
            Ok(Self::Legacy(SoundFormat::from(n as u8)))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OnMetaDataVideoCodecId {
    Legacy(VideoCodecId),
    Enhanced(VideoFourCc),
}

impl OnMetaDataVideoCodecId {
    fn from_amf0(value: &Amf0Value<'_>) -> Result<Self, Error> {
        let n = value.as_number()? as u32;

        // Since VideoCodecId is a u8, we can be sure that the number represents an VideoFourCc if it is greater
        // than u8::MAX.
        // Additionally, since the smallest possible VideoFourCc (4 spaces) is greater than u8::MAX,
        // we can be sure that the number cannot represent an VideoFourCc if it is smaller than u8::MAX.
        if n > u8::MAX as u32 {
            Ok(Self::Enhanced(VideoFourCc::from(n.to_be_bytes())))
        } else {
            Ok(Self::Legacy(VideoCodecId::from(n as u8)))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnMetaData {
    pub audiocodecid: Option<OnMetaDataAudioCodecId>,
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
    pub videocodecid: Option<OnMetaDataVideoCodecId>,
    pub videodatarate: Option<f64>,
    pub width: Option<f64>,
    pub audio_track_id_info_map: Option<HashMap<String, Amf0Value<'static>>>,
    pub video_track_id_info_map: Option<HashMap<String, Amf0Value<'static>>>,
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
        let mut audio_track_id_info_map = None;
        let mut video_track_id_info_map = None;

        for (key, value) in value.iter() {
            match key.as_ref() {
                "audiocodecid" => audiocodecid = Some(OnMetaDataAudioCodecId::from_amf0(value)?),
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
                "videocodecid" => videocodecid = Some(OnMetaDataVideoCodecId::from_amf0(value)?),
                "videodatarate" => videodatarate = Some(value.as_number()?),
                "width" => width = Some(value.as_number()?),
                "audioTrackIdInfoMap" => {
                    let mut map = HashMap::new();

                    let object = value.as_object()?;
                    for (key, value) in object.iter() {
                        map.insert(key.to_string(), value.to_owned());
                    }

                    audio_track_id_info_map = Some(map);
                }
                "videoTrackIdInfoMap" => {
                    let mut map = HashMap::new();

                    let object = value.as_object()?;
                    for (key, value) in object.iter() {
                        map.insert(key.to_string(), value.to_owned());
                    }

                    video_track_id_info_map = Some(map);
                }
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
            audio_track_id_info_map,
            video_track_id_info_map,
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
    // Boxed because it's so big
    OnMetaData(Box<OnMetaData>),
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

                let Amf0Value::Object(data) = value else { unreachable!() };
                let data = OnMetaData::try_from(data)?;

                Ok(Self::OnMetaData(Box::new(data)))
            }
            "onXMPData" => {
                let value = amf0_reader.decode()?;

                let Amf0Value::Object(data) = value else { unreachable!() };
                let data = OnXmpData::try_from(data)?;

                Ok(Self::OnXmpData(data))
            }
            _ => {
                tracing::trace!(name = %name, "unknown script data");

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
        #[rustfmt::skip]
        let data = [
            Amf0Marker::String as u8,
            0, 10, // Length (10 bytes)
            b'o', b'n', b'M', b'e', b't', b'a', b'D', b'a', b't', b'a',// "onMetaData"
            Amf0Marker::Object as u8,
            0, 5, // Length (5 bytes)
            b'w', b'i', b'd', b't', b'h', // "width"
            Amf0Marker::Number as u8,
            width[0],
            width[1],
            width[2],
            width[3],
            width[4],
            width[5],
            width[6],
            width[7],
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let mut reader = io::Cursor::new(Bytes::from_owner(data));

        let script_data = ScriptData::demux(&mut reader).unwrap();

        let ScriptData::OnMetaData(metadata) = script_data else {
            panic!("expected onMetaData");
        };

        assert_eq!(
            *metadata,
            OnMetaData {
                audiocodecid: None,
                audiodatarate: None,
                audiodelay: None,
                audiosamplerate: None,
                audiosamplesize: None,
                can_seek_to_end: None,
                creationdate: None,
                duration: None,
                filesize: None,
                framerate: None,
                height: None,
                stereo: None,
                videocodecid: None,
                videodatarate: None,
                width: Some(1280.0),
                audio_track_id_info_map: None,
                video_track_id_info_map: None,
                other: HashMap::new(),
            }
        );
    }
}
