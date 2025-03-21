//! Types and functions for working with metadata video packets.

use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Object, Amf0Value};

use crate::error::Error;

/// Color configuration metadata.
///
/// > `colorPrimaries`, `transferCharacteristics` and `matrixCoefficients` are defined
/// > in ISO/IEC 23091-4/ITU-T H.273. The values are an index into
/// > respective tables which are described in "Colour primaries",
/// > "Transfer characteristics" and "Matrix coefficients" sections.
/// > It is RECOMMENDED to provide these values.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataColorInfo {
    /// Color configuration metadata.
    pub color_config: Option<MetadataColorInfoColorConfig>,
    /// HDR content light level metadata.
    pub hdr_cll: Option<MetadataColorInfoHdrCll>,
    /// HDR mastering display color volume metadata.
    pub hdr_mdcv: Option<MetadataColorInfoHdrMdcv>,
}

// Be warned: Insanely ugly code ahead
// We should maybe implement serde support in the amf0 crate

impl TryFrom<Amf0Object<'_>> for MetadataColorInfo {
    type Error = Error;

    fn try_from(value: Amf0Object<'_>) -> Result<Self, Self::Error> {
        let mut color_config = None;
        let mut hdr_cll = None;
        let mut hdr_mdcv = None;

        for (key, value) in value.iter() {
            match key.as_ref() {
                "colorConfig" => {
                    let color_config_object = value.as_object()?;

                    let mut bit_depth = None;
                    let mut color_primaries = None;
                    let mut transfer_characteristics = None;
                    let mut matrix_coefficients = None;

                    for (key, value) in color_config_object.iter() {
                        match key.as_ref() {
                            "bitDepth" => {
                                bit_depth = Some(value.as_number()?);
                            }
                            "colorPrimaries" => {
                                color_primaries = Some(value.as_number()?);
                            }
                            "transferCharacteristics" => {
                                transfer_characteristics = Some(value.as_number()?);
                            }
                            "matrixCoefficients" => {
                                matrix_coefficients = Some(value.as_number()?);
                            }
                            _ => {}
                        }
                    }

                    color_config = Some(MetadataColorInfoColorConfig {
                        bit_depth,
                        color_primaries,
                        transfer_characteristics,
                        matrix_coefficients,
                    });
                }
                "hdrCll" => {
                    let hdr_cll_object = value.as_object()?;

                    let mut max_fall = None;
                    let mut max_cll = None;

                    for (key, value) in hdr_cll_object.iter() {
                        match key.as_ref() {
                            "maxFall" => {
                                max_fall = Some(value.as_number()?);
                            }
                            "maxCll" => {
                                max_cll = Some(value.as_number()?);
                            }
                            _ => {}
                        }
                    }

                    hdr_cll = Some(MetadataColorInfoHdrCll { max_fall, max_cll });
                }
                "hdrMdcv" => {
                    let hdr_mdcv_object = value.as_object()?;

                    let mut red_x = None;
                    let mut red_y = None;
                    let mut green_x = None;
                    let mut green_y = None;
                    let mut blue_x = None;
                    let mut blue_y = None;
                    let mut white_point_x = None;
                    let mut white_point_y = None;
                    let mut max_luminance = None;
                    let mut min_luminance = None;

                    for (key, value) in hdr_mdcv_object.iter() {
                        match key.as_ref() {
                            "redX" => {
                                red_x = Some(value.as_number()?);
                            }
                            "redY" => {
                                red_y = Some(value.as_number()?);
                            }
                            "greenX" => {
                                green_x = Some(value.as_number()?);
                            }
                            "greenY" => {
                                green_y = Some(value.as_number()?);
                            }
                            "blueX" => {
                                blue_x = Some(value.as_number()?);
                            }
                            "blueY" => {
                                blue_y = Some(value.as_number()?);
                            }
                            "whitePointX" => {
                                white_point_x = Some(value.as_number()?);
                            }
                            "whitePointY" => {
                                white_point_y = Some(value.as_number()?);
                            }
                            "maxLuminance" => {
                                max_luminance = Some(value.as_number()?);
                            }
                            "minLuminance" => {
                                min_luminance = Some(value.as_number()?);
                            }
                            _ => {}
                        }
                    }

                    hdr_mdcv = Some(MetadataColorInfoHdrMdcv {
                        red_x,
                        red_y,
                        green_x,
                        green_y,
                        blue_x,
                        blue_y,
                        white_point_x,
                        white_point_y,
                        max_luminance,
                        min_luminance,
                    });
                }
                _ => {}
            }
        }

        Ok(MetadataColorInfo {
            color_config,
            hdr_cll,
            hdr_mdcv,
        })
    }
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
        object: Vec<(String, Amf0Value<'static>)>,
    },
}

impl VideoPacketMetadataEntry {
    /// Read a video packet metadata entry from the given [`Amf0Decoder`].
    pub fn read(reader: &mut Amf0Decoder<'_>) -> Result<Self, Error> {
        let Amf0Value::String(key) = reader.decode_with_type(Amf0Marker::String)? else {
            unreachable!()
        };

        let Amf0Value::Object(value) = reader.decode_with_type(Amf0Marker::Object)? else {
            unreachable!()
        };

        match key.as_ref() {
            "colorInfo" => Ok(VideoPacketMetadataEntry::ColorInfo(MetadataColorInfo::try_from(value)?)),
            _ => {
                let object = value
                    .into_owned()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_owned()))
                    .collect();

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
    use std::borrow::Cow;

    use scuffle_amf0::{Amf0Decoder, Amf0Object, Amf0Value};

    use super::VideoPacketMetadataEntry;
    use crate::video::body::enhanced::metadata::MetadataColorInfo;

    #[test]
    fn metadata_color_info() {
        let object: Amf0Object = Cow::Owned(vec![
            (
                "colorConfig".into(),
                Amf0Value::Object(Cow::Owned(vec![
                    ("bitDepth".into(), 10.0.into()),
                    ("colorPrimaries".into(), 1.0.into()),
                    ("transferCharacteristics".into(), 1.0.into()),
                    ("matrixCoefficients".into(), 1.0.into()),
                ])),
            ),
            (
                "hdrCll".into(),
                Amf0Value::Object(Cow::Owned(vec![
                    ("maxFall".into(), 1000.0.into()),
                    ("maxCll".into(), 1000.0.into()),
                ])),
            ),
            (
                "hdrMdcv".into(),
                Amf0Value::Object(Cow::Owned(vec![
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
                ])),
            ),
        ]);

        let mut buf = Vec::new();
        scuffle_amf0::Amf0Encoder::encode_string(&mut buf, "colorInfo").unwrap();
        scuffle_amf0::Amf0Encoder::encode_object(&mut buf, &object).unwrap();

        let mut reader = Amf0Decoder::new(&buf);

        let entry = VideoPacketMetadataEntry::read(&mut reader).unwrap();

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
