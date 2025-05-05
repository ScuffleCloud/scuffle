//! Script data structures

use core::fmt;
use std::io;

use bytes::Bytes;
use scuffle_amf0::de::MultiValue;
use scuffle_amf0::decoder::Amf0Decoder;
use scuffle_amf0::{Amf0Object, Amf0Value};
use scuffle_bytes_util::{BytesCursorExt, StringCow};
use serde::de::VariantAccess;
use serde_derive::Deserialize;

use crate::audio::header::enhanced::AudioFourCc;
use crate::audio::header::legacy::SoundFormat;
use crate::error::FlvError;
use crate::video::header::enhanced::VideoFourCc;
use crate::video::header::legacy::VideoCodecId;

/// FLV `onMetaData` audio codec ID.
///
/// Either a legacy [`SoundFormat`] or an enhanced [`AudioFourCc`].
/// Appears as `audiocodecid` in the [`OnMetaData`] script data.
#[derive(Debug, Clone, PartialEq)]
pub enum OnMetaDataAudioCodecId {
    /// Legacy audio codec ID.
    Legacy(SoundFormat),
    /// Enhanced audio codec ID.
    Enhanced(AudioFourCc),
}

impl<'de> serde::Deserialize<'de> for OnMetaDataAudioCodecId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let n: u32 = serde::Deserialize::deserialize(deserializer)?;

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

/// FLV `onMetaData` video codec ID.
///
/// Either a legacy [`VideoCodecId`] or an enhanced [`VideoFourCc`].
/// Appears as `videocodecid` in the [`OnMetaData`] script data.
#[derive(Debug, Clone, PartialEq)]
pub enum OnMetaDataVideoCodecId {
    /// Legacy video codec ID.
    Legacy(VideoCodecId),
    /// Enhanced video codec ID.
    Enhanced(VideoFourCc),
}

impl<'de> serde::Deserialize<'de> for OnMetaDataVideoCodecId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let n: u32 = serde::Deserialize::deserialize(deserializer)?;

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

/// FLV `onMetaData` script data
///
/// Defined by:
/// - Legacy FLV spec, Annex E.5
/// - Enhanced RTMP spec, page 13-16, Enhancing onMetaData
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", bound = "'a: 'de")]
pub struct OnMetaData<'a> {
    /// Audio codec ID used in the file.
    #[serde(default)]
    pub audiocodecid: Option<OnMetaDataAudioCodecId>,
    /// Audio bitrate, in kilobits per second.
    #[serde(default)]
    pub audiodatarate: Option<f64>,
    /// Delay introduced by the audio codec, in seconds.
    #[serde(default)]
    pub audiodelay: Option<f64>,
    /// Frequency at which the audio stream is replayed.
    #[serde(default)]
    pub audiosamplerate: Option<f64>,
    /// Resolution of a single audio sample.
    #[serde(default)]
    pub audiosamplesize: Option<f64>,
    /// Indicating the last video frame is a key frame.
    #[serde(default)]
    pub can_seek_to_end: Option<bool>,
    /// Creation date and time.
    #[serde(default)]
    pub creationdate: Option<String>,
    /// Total duration of the file, in seconds.
    #[serde(default)]
    pub duration: Option<f64>,
    /// Total size of the file, in bytes.
    #[serde(default)]
    pub filesize: Option<f64>,
    /// Number of frames per second.
    #[serde(default)]
    pub framerate: Option<f64>,
    /// Height of the video, in pixels.
    #[serde(default)]
    pub height: Option<f64>,
    /// Indicates stereo audio.
    #[serde(default)]
    pub stereo: Option<bool>,
    /// Video codec ID used in the file.
    #[serde(default)]
    pub videocodecid: Option<OnMetaDataVideoCodecId>,
    /// Video bitrate, in kilobits per second.
    #[serde(default)]
    pub videodatarate: Option<f64>,
    /// Width of the video, in pixels.
    #[serde(default)]
    pub width: Option<f64>,
    /// The audioTrackIdInfoMap and videoTrackIdInfoMap objects are designed to store
    /// metadata for audio and video tracks respectively. Each object uses a TrackId as
    /// a key to map to properties that detail the unique characteristics of each
    /// individual track, diverging from the default configurations.
    ///
    /// Key-Value Structure:
    /// - Keys: Each TrackId acts as a unique identifier for a specific audio or video track.
    /// - Values: Track Objects containing metadata that specify characteristics which deviate from the default track settings.
    ///
    /// Properties of Each Track Object:
    /// - These properties detail non-standard configurations needed for
    ///   custom handling of the track, facilitating specific adjustments
    ///   to enhance track performance and quality for varied conditions.
    /// - For videoTrackIdInfoMap:
    ///   - Properties such as width, height, videodatarate, etc.
    ///     specify video characteristics that differ from standard
    ///     settings.
    /// - For audioTrackIdInfoMap:
    ///   - Properties such as audiodatarate, channels, etc., define
    ///     audio characteristics that differ from standard
    ///     configurations.
    ///
    /// Purpose:
    /// - The purpose of these maps is to specify unique properties for
    ///   each track, ensuring tailored configurations that optimize
    ///   performance and quality for specific media content and delivery
    ///   scenarios.
    ///
    /// This structure provides a framework for detailed customization and control over
    /// the media tracks, ensuring optimal management and delivery across various types
    /// of content and platforms.
    #[serde(default, borrow)]
    pub audio_track_id_info_map: Option<Amf0Object<'a>>,
    /// See [`OnMetaData::audio_track_id_info_map`].
    #[serde(default, borrow)]
    pub video_track_id_info_map: Option<Amf0Object<'a>>,
    /// Any other metadata contained in the script data.
    #[serde(flatten, borrow)]
    pub other: Amf0Object<'a>,
}

/// XMP Metadata
///
/// Defined by:
/// - Legacy FLV spec, Annex E.6
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", bound = "'a: 'de")]
pub struct OnXmpData<'a> {
    /// XMP metadata, formatted according to the XMP metadata specification.
    ///
    /// For further details, see [www.adobe.com/devnet/xmp/pdfs/XMPSpecificationPart3.pdf](https://web.archive.org/web/20090306165322/https://www.adobe.com/devnet/xmp/pdfs/XMPSpecificationPart3.pdf).
    #[serde(default, rename = "liveXML")]
    live_xml: Option<StringCow<'a>>,
    /// Any other metadata contained in the script data.
    #[serde(flatten, borrow)]
    other: Amf0Object<'a>,
}

/// FLV `SCRIPTDATA` tag
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.4.1
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptData<'a> {
    /// `onMetaData` script data.
    ///
    /// Boxed because it's so big.
    OnMetaData(Box<OnMetaData<'a>>),
    /// `onXMPData` script data.
    OnXmpData(OnXmpData<'a>),
    /// Any other script data.
    Other {
        /// The name of the script data.
        name: StringCow<'a>,
        /// The data of the script data.
        data: Vec<Amf0Value<'static>>,
    },
}

impl<'de> serde::Deserialize<'de> for ScriptData<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        const SCRIPT_DATA: &str = "ScriptData";
        const ON_META_DATA: &str = "onMetaData";
        const ON_XMP_DATA: &str = "onXMPData";

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ScriptData<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(SCRIPT_DATA)
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                let (name, content): (StringCow<'de>, A::Variant) = data.variant()?;

                match name.as_ref() {
                    ON_META_DATA => Ok(ScriptData::OnMetaData(Box::new(content.newtype_variant()?))),
                    ON_XMP_DATA => Ok(ScriptData::OnXmpData(content.newtype_variant()?)),
                    _ => Ok(ScriptData::Other {
                        name,
                        data: content
                            .newtype_variant::<MultiValue<Vec<Amf0Value>>>()?
                            .0
                            .into_iter()
                            .map(|v| v.into_owned())
                            .collect(),
                    }),
                }
            }
        }

        deserializer.deserialize_enum(SCRIPT_DATA, &[ON_META_DATA, ON_XMP_DATA], Visitor)
    }
}

impl ScriptData<'_> {
    /// Demux the [`ScriptData`] from the given reader.
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> Result<Self, FlvError> {
        let buf = reader.extract_remaining();
        let mut decoder = Amf0Decoder::from_buf(buf);

        serde::de::Deserialize::deserialize(&mut decoder).map_err(FlvError::Amf0)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_amf0::Amf0Marker;
    use scuffle_amf0::encoder::Amf0Encoder;

    use super::*;

    #[test]
    fn script_on_meta_data() {
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
                other: Amf0Object::new(),
            }
        );
    }

    #[test]
    fn script_on_meta_data_full() {
        let mut data = Vec::new();
        let mut encoder = Amf0Encoder::new(&mut data);

        let audio_track_id_info_map = [("test".into(), Amf0Value::Number(1.0))].into_iter().collect();
        let video_track_id_info_map = [("test2".into(), Amf0Value::Number(2.0))].into_iter().collect();

        encoder.encode_string("onMetaData").unwrap();
        let object: Amf0Object = [
            (
                "audiocodecid".into(),
                Amf0Value::Number(u32::from_be_bytes(AudioFourCc::Aac.0) as f64),
            ),
            ("audiodatarate".into(), Amf0Value::Number(128.0)),
            ("audiodelay".into(), Amf0Value::Number(0.0)),
            ("audiosamplerate".into(), Amf0Value::Number(44100.0)),
            ("audiosamplesize".into(), Amf0Value::Number(16.0)),
            ("canSeekToEnd".into(), Amf0Value::Boolean(true)),
            ("creationdate".into(), Amf0Value::String("2025-01-01T00:00:00Z".into())),
            ("duration".into(), Amf0Value::Number(60.0)),
            ("filesize".into(), Amf0Value::Number(1024.0)),
            ("framerate".into(), Amf0Value::Number(30.0)),
            ("height".into(), Amf0Value::Number(720.0)),
            ("stereo".into(), Amf0Value::Boolean(true)),
            (
                "videocodecid".into(),
                Amf0Value::Number(u32::from_be_bytes(VideoFourCc::Avc.0) as f64),
            ),
            ("videodatarate".into(), Amf0Value::Number(1024.0)),
            ("width".into(), Amf0Value::Number(1280.0)),
            ("audioTrackIdInfoMap".into(), Amf0Value::Object(audio_track_id_info_map)),
            ("videoTrackIdInfoMap".into(), Amf0Value::Object(video_track_id_info_map)),
        ]
        .into_iter()
        .collect();
        encoder.encode_object(&object).unwrap();

        let mut reader = io::Cursor::new(Bytes::from_owner(data));
        let script_data = ScriptData::demux(&mut reader).unwrap();

        let ScriptData::OnMetaData(metadata) = script_data else {
            panic!("expected onMetaData");
        };

        assert_eq!(
            *metadata,
            OnMetaData {
                audiocodecid: Some(OnMetaDataAudioCodecId::Enhanced(AudioFourCc::Aac)),
                audiodatarate: Some(128.0),
                audiodelay: Some(0.0),
                audiosamplerate: Some(44100.0),
                audiosamplesize: Some(16.0),
                can_seek_to_end: Some(true),
                creationdate: Some("2025-01-01T00:00:00Z".to_string()),
                duration: Some(60.0),
                filesize: Some(1024.0),
                framerate: Some(30.0),
                height: Some(720.0),
                stereo: Some(true),
                videocodecid: Some(OnMetaDataVideoCodecId::Enhanced(VideoFourCc::Avc)),
                videodatarate: Some(1024.0),
                width: Some(1280.0),
                audio_track_id_info_map: Some([("test".into(), Amf0Value::Number(1.0))].into_iter().collect()),
                video_track_id_info_map: Some([("test2".into(), Amf0Value::Number(2.0))].into_iter().collect()),
                other: Amf0Object::new(),
            }
        );
    }

    #[test]
    fn script_on_xmp_data() {
        #[rustfmt::skip]
        let data = [
            Amf0Marker::String as u8,
            0, 9, // Length (9 bytes)
            b'o', b'n', b'X', b'M', b'P', b'D', b'a', b't', b'a',// "onXMPData"
            Amf0Marker::Object as u8,
            0, 7, // Length (7 bytes)
            b'l', b'i', b'v', b'e', b'X', b'M', b'L', // "liveXML"
            Amf0Marker::String as u8,
            0, 5, // Length (5 bytes)
            b'h', b'e', b'l', b'l', b'o', // "hello"
            0, 4, // Length (7 bytes)
            b't', b'e', b's', b't', // "test"
            Amf0Marker::Null as u8,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let mut reader = io::Cursor::new(Bytes::from_owner(data));

        let script_data = ScriptData::demux(&mut reader).unwrap();

        let ScriptData::OnXmpData(xmp_data) = script_data else {
            panic!("expected onXMPData");
        };

        assert_eq!(
            xmp_data,
            OnXmpData {
                live_xml: Some("hello".into()),
                other: [("test".into(), Amf0Value::Null)].into_iter().collect(),
            }
        );
    }

    #[test]
    fn script_other() {
        #[rustfmt::skip]
        let data = [
            Amf0Marker::String as u8,
            0, 10, // Length (10 bytes)
            b'o', b'n', b'W', b'h', b'a', b't', b'e', b'v', b'e', b'r',// "onWhatever"
            Amf0Marker::Object as u8,
            0, 4, // Length (4 bytes)
            b't', b'e', b's', b't', // "test"
            Amf0Marker::String as u8,
            0, 5, // Length (5 bytes)
            b'h', b'e', b'l', b'l', b'o', // "hello"
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let mut reader = io::Cursor::new(Bytes::from_owner(data));

        let script_data = ScriptData::demux(&mut reader).unwrap();

        let ScriptData::Other { name, data } = script_data else {
            panic!("expected onXMPData");
        };

        let object: Amf0Object = [("test".into(), Amf0Value::String("hello".into()))].into_iter().collect();

        assert_eq!(name, "onWhatever");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], Amf0Value::Object(object));
    }
}
