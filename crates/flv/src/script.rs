//! Script data structures

use std::collections::HashMap;
use std::io;

use bytes::Bytes;
use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Object, Amf0Value};
use scuffle_bytes_util::BytesCursorExt;

use crate::audio::header::enhanced::AudioFourCc;
use crate::audio::header::legacy::SoundFormat;
use crate::error::Error;
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

impl OnMetaDataAudioCodecId {
    /// Read the audio codec ID from the given AMF0 value.
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

impl OnMetaDataVideoCodecId {
    /// Read the video codec ID from the given AMF0 value.
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

/// FLV `onMetaData` script data
///
/// Defined by:
/// - Legacy FLV spec, Annex E.5
/// - Enhanced RTMP spec, page 13-16, Enhancing onMetaData
#[derive(Debug, Clone, PartialEq)]
pub struct OnMetaData {
    /// Audio codec ID used in the file.
    pub audiocodecid: Option<OnMetaDataAudioCodecId>,
    /// Audio bitrate, in kilobits per second.
    pub audiodatarate: Option<f64>,
    /// Delay introduced by the audio codec, in seconds.
    pub audiodelay: Option<f64>,
    /// Frequency at which the audio stream is replayed.
    pub audiosamplerate: Option<f64>,
    /// Resolution of a single audio sample.
    pub audiosamplesize: Option<f64>,
    /// Indicating the last video frame is a key frame.
    pub can_seek_to_end: Option<bool>,
    /// Creation date and time.
    pub creationdate: Option<String>,
    /// Total duration of the file, in seconds.
    pub duration: Option<f64>,
    /// Total size of the file, in bytes.
    pub filesize: Option<f64>,
    /// Number of frames per second.
    pub framerate: Option<f64>,
    /// Height of the video, in pixels.
    pub height: Option<f64>,
    /// Indicates stereo audio.
    pub stereo: Option<bool>,
    /// Video codec ID used in the file.
    pub videocodecid: Option<OnMetaDataVideoCodecId>,
    /// Video bitrate, in kilobits per second.
    pub videodatarate: Option<f64>,
    /// Width of the video, in pixels.
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
    pub audio_track_id_info_map: Option<HashMap<String, Amf0Value<'static>>>,
    /// See [`OnMetaData::audio_track_id_info_map`].
    pub video_track_id_info_map: Option<HashMap<String, Amf0Value<'static>>>,
    /// Any other metadata contained in the script data.
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

/// XMP Metadata
///
/// Defined by:
/// - Legacy FLV spec, Annex E.6
#[derive(Debug, Clone, PartialEq)]
pub struct OnXmpData {
    /// XMP metadata, formatted according to the XMP metadata specification.
    ///
    /// For further details, see [www.adobe.com/devnet/xmp/pdfs/XMPSpecificationPart3.pdf](https://web.archive.org/web/20090306165322/https://www.adobe.com/devnet/xmp/pdfs/XMPSpecificationPart3.pdf).
    live_xml: Option<String>,
    /// Any other metadata contained in the script data.
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

/// FLV `SCRIPTDATA` tag
///
/// Defined by:
/// - Legacy FLV spec, Annex E.4.4.1
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptData {
    /// `onMetaData` script data.
    ///
    /// Boxed because it's so big.
    OnMetaData(Box<OnMetaData>),
    /// `onXMPData` script data.
    OnXmpData(OnXmpData),
    /// Any other script data.
    Other {
        /// The name of the script data.
        name: String,
        /// The data of the script data.
        data: Vec<Amf0Value<'static>>,
    },
}

impl ScriptData {
    /// Demux the [`ScriptData`] from the given reader.
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
    use std::borrow::Cow;

    use scuffle_amf0::Amf0Encoder;

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
                other: HashMap::new(),
            }
        );
    }

    #[test]
    fn script_on_meta_data_full() {
        let mut data = Vec::new();

        let audio_track_id_info_map = vec![("test".into(), Amf0Value::Number(1.0))].into();
        let video_track_id_info_map = vec![("test2".into(), Amf0Value::Number(2.0))].into();

        Amf0Encoder::encode_string(&mut data, "onMetaData").unwrap();
        Amf0Encoder::encode_object(
            &mut data,
            &[
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
            ],
        )
        .unwrap();

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
                audio_track_id_info_map: Some([("test".to_string(), Amf0Value::Number(1.0))].into()),
                video_track_id_info_map: Some([("test2".to_string(), Amf0Value::Number(2.0))].into()),
                other: HashMap::new(),
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
                live_xml: Some("hello".to_string()),
                other: [("test".to_string(), Amf0Value::Null)].into(),
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

        let object: Amf0Object = vec![(Cow::Borrowed("test"), Amf0Value::String(Cow::Borrowed("hello")))].into();

        assert_eq!(name, "onWhatever");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], Amf0Value::Object(object));
    }
}
