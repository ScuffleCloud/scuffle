//! Types and functions for working with metadata video packets.

use std::io;

use scuffle_amf0::Amf0Object;
use serde::Deserialize;

use crate::error::FlvError;

/// Color configuration metadata.
///
/// > `colorPrimaries`, `transferCharacteristics` and `matrixCoefficients` are defined
/// > in ISO/IEC 23091-4/ITU-T H.273. The values are an index into
/// > respective tables which are described in "Colour primaries",
/// > "Transfer characteristics" and "Matrix coefficients" sections.
/// > It is RECOMMENDED to provide these values.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataColorInfoColorConfig {
    /// Number of bits used to record the color channels for each pixel.
    ///
    /// SHOULD be 8, 10 or 12
    pub bit_depth: Option<f64>,
    /// Indicates the chromaticity coordinates of the source color primaries.
    ///
    /// enumeration [0-255]
    pub color_primaries: Option<f64>,
    /// Opto-electronic transfer characteristic function (e.g., PQ, HLG).
    ///
    /// enumeration [0-255]
    pub transfer_characteristics: Option<f64>,
    /// Matrix coefficients used in deriving luma and chroma signals.
    ///
    /// enumeration [0-255]
    pub matrix_coefficients: Option<f64>,
}

/// HDR content light level metadata.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataColorInfoHdrCll {
    /// Maximum value of the frame average light level
    /// (in 1 cd/m2) of the entire playback sequence.
    ///
    /// [0.0001-10000]
    pub max_fall: Option<f64>,
    /// Maximum light level of any single pixel (in 1 cd/m2)
    /// of the entire playback sequence.
    ///
    /// [0.0001-10000]
    pub max_cll: Option<f64>,
}

/// HDR mastering display color volume metadata.
///
/// > The hdrMdcv object defines mastering display (i.e., where
/// > creative work is done during the mastering process) color volume (a.k.a., mdcv)
/// > metadata which describes primaries, white point and min/max luminance. The
/// > hdrMdcv object SHOULD be provided.
/// >
/// > Specification of the metadata along with its ranges adhere to the
/// > ST 2086:2018 - SMPTE Standard (except for minLuminance see
/// > comments below)
///
/// > Mastering display color volume (mdcv) xy Chromaticity Coordinates within CIE
/// > 1931 color space.
///
/// > Values SHALL be specified with four decimal places. The x coordinate SHALL
/// > be in the range [0.0001, 0.7400]. The y coordinate SHALL be
/// > in the range [0.0001, 0.8400].
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataColorInfoHdrMdcv {
    /// Red x coordinate.
    pub red_x: Option<f64>,
    /// Red y coordinate.
    pub red_y: Option<f64>,
    /// Green x coordinate.
    pub green_x: Option<f64>,
    /// Green y coordinate.
    pub green_y: Option<f64>,
    /// Blue x coordinate.
    pub blue_x: Option<f64>,
    /// Blue y coordinate.
    pub blue_y: Option<f64>,
    /// White point x coordinate.
    pub white_point_x: Option<f64>,
    /// White point y coordinate.
    pub white_point_y: Option<f64>,
    /// Max display luminance of the mastering display (in 1 cd/m2 ie. nits).
    ///
    /// > note: ST 2086:2018 - SMPTE Standard specifies minimum display mastering
    /// > luminance in multiples of 0.0001 cd/m2.
    ///
    /// > For consistency we specify all values
    /// > in 1 cd/m2. Given that a hypothetical perfect screen has a peak brightness
    /// > of 10,000 nits and a black level of .0005 nits we do not need to
    /// > switch units to 0.0001 cd/m2 to increase resolution on the lower end of the
    /// > minLuminance property. The ranges (in nits) mentioned below suffice
    /// > the theoretical limit for Mastering Reference Displays and adhere to the
    /// > SMPTE ST 2084 standard (a.k.a., PQ) which is capable of representing full gamut
    /// > of luminance level.
    pub max_luminance: Option<f64>,
    /// Min display luminance of the mastering display (in 1 cd/m2 ie. nits).
    ///
    /// See [`max_luminance`](MetadataColorInfoHdrMdcv::max_luminance) for details.
    pub min_luminance: Option<f64>,
}

/// Color info metadata.
///
/// Defined by:
/// - Enhanced RTMP spec, page 32-34, Metadata Frame
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataColorInfo {
    /// Color configuration metadata.
    pub color_config: Option<MetadataColorInfoColorConfig>,
    /// HDR content light level metadata.
    pub hdr_cll: Option<MetadataColorInfoHdrCll>,
    /// HDR mastering display color volume metadata.
    pub hdr_mdcv: Option<MetadataColorInfoHdrMdcv>,
}

/// A single entry in a metadata video packet.
// It will almost always be ColorInfo, so it's fine that it wastes space when it's the other variant
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketMetadataEntry {
    /// Color info metadata.
    ColorInfo(MetadataColorInfo),
    /// Any other metadata entry.
    Other {
        /// The key of the metadata entry.
        key: String,
        /// The metadata object.
        object: Amf0Object,
    },
}

impl VideoPacketMetadataEntry {
    /// Read a video packet metadata entry from the given [`scuffle_amf0::Deserializer`].
    pub fn read<R: io::Read + io::Seek>(deserializer: &mut scuffle_amf0::Deserializer<R>) -> Result<Self, FlvError> {
        let key = String::deserialize(&mut *deserializer)?;

        match key.as_ref() {
            "colorInfo" => Ok(VideoPacketMetadataEntry::ColorInfo(MetadataColorInfo::deserialize(
                deserializer,
            )?)),
            _ => {
                let object = Amf0Object::deserialize(deserializer)?;

                Ok(VideoPacketMetadataEntry::Other {
                    key: key.to_string(),
                    object,
                })
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::io;

    use scuffle_amf0::{Amf0Object, Amf0Value};
    use serde::Serialize;

    use super::VideoPacketMetadataEntry;
    use crate::video::body::enhanced::metadata::MetadataColorInfo;

    #[test]
    fn metadata_color_info() {
        let object: Amf0Object = [
            (
                "colorConfig".into(),
                Amf0Value::Object(
                    [
                        ("bitDepth".into(), 10.0.into()),
                        ("colorPrimaries".into(), 1.0.into()),
                        ("transferCharacteristics".into(), 1.0.into()),
                        ("matrixCoefficients".into(), 1.0.into()),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ),
            (
                "hdrCll".into(),
                Amf0Value::Object(
                    [("maxFall".into(), 1000.0.into()), ("maxCll".into(), 1000.0.into())]
                        .into_iter()
                        .collect(),
                ),
            ),
            (
                "hdrMdcv".into(),
                Amf0Value::Object(
                    [
                        ("redX".into(), 0.0.into()),
                        ("redY".into(), 0.0.into()),
                        ("greenX".into(), 0.0.into()),
                        ("greenY".into(), 0.0.into()),
                        ("blueX".into(), 0.0.into()),
                        ("blueY".into(), 0.0.into()),
                        ("whitePointX".into(), 0.0.into()),
                        ("whitePointY".into(), 0.0.into()),
                        ("maxLuminance".into(), 0.0.into()),
                        ("minLuminance".into(), 0.0.into()),
                    ]
                    .into_iter()
                    .collect(),
                ),
            ),
        ]
        .into_iter()
        .collect();

        let mut buf = Vec::new();
        let mut serializer = scuffle_amf0::Serializer::new(&mut buf);
        "colorInfo".serialize(&mut serializer).unwrap();
        object.serialize(&mut serializer).unwrap();

        let mut deserializer = scuffle_amf0::Deserializer::new(io::Cursor::new(buf));
        let entry = VideoPacketMetadataEntry::read(&mut deserializer).unwrap();

        assert_eq!(
            entry,
            VideoPacketMetadataEntry::ColorInfo(MetadataColorInfo {
                color_config: Some(super::MetadataColorInfoColorConfig {
                    bit_depth: Some(10.0),
                    color_primaries: Some(1.0),
                    transfer_characteristics: Some(1.0),
                    matrix_coefficients: Some(1.0),
                }),
                hdr_cll: Some(super::MetadataColorInfoHdrCll {
                    max_fall: Some(1000.0),
                    max_cll: Some(1000.0),
                }),
                hdr_mdcv: Some(super::MetadataColorInfoHdrMdcv {
                    red_x: Some(0.0),
                    red_y: Some(0.0),
                    green_x: Some(0.0),
                    green_y: Some(0.0),
                    blue_x: Some(0.0),
                    blue_y: Some(0.0),
                    white_point_x: Some(0.0),
                    white_point_y: Some(0.0),
                    max_luminance: Some(0.0),
                    min_luminance: Some(0.0),
                }),
            })
        )
    }
}
