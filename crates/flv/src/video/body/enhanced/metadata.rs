use scuffle_amf0::{Amf0Decoder, Amf0Marker, Amf0Object, Amf0Value};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataColorInfoColorConfig {
    pub bit_depth: f64,
    pub color_space: f64,
    pub color_primaries: f64,
    pub transfer_characteristics: f64,
    pub matrix_coefficients: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataColorInfoHdrCll {
    pub max_fall: f64,
    pub max_cll: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataColorInfoHdrMdcv {
    pub red_x: f64,
    pub red_y: f64,
    pub green_x: f64,
    pub green_y: f64,
    pub blue_x: f64,
    pub blue_y: f64,
    pub white_point_x: f64,
    pub white_point_y: f64,
    pub max_luminance: f64,
    pub min_luminance: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataColorInfo {
    pub color_config: MetadataColorInfoColorConfig,
    pub hdr_cll: MetadataColorInfoHdrCll,
    pub hdr_mdcv: MetadataColorInfoHdrMdcv,
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataColorInfoError {
    #[error("unexpected type, expected {expected:?}, found {found:?}")]
    UnexpectedType { expected: Amf0Marker, found: Amf0Marker },
    #[error("missing field {0}")]
    MissingField(&'static str),
}

// Be warned: Insanely ugly code ahead
// We should maybe implement serde support in the amf0 crate

impl TryFrom<Amf0Object<'_>> for MetadataColorInfo {
    type Error = MetadataColorInfoError;

    fn try_from(value: Amf0Object<'_>) -> Result<Self, Self::Error> {
        // let value = value.into_owned();
        let mut color_config = None;
        let mut hdr_cll = None;
        let mut hdr_mdcv = None;

        for (key, value) in value.iter() {
            match key.as_ref() {
                "colorConfig" => {
                    let Amf0Value::Object(color_config_object) = value else {
                        return Err(MetadataColorInfoError::UnexpectedType {
                            expected: Amf0Marker::Object,
                            found: value.marker(),
                        });
                    };

                    let mut bit_depth = None;
                    let mut color_space = None;
                    let mut color_primaries = None;
                    let mut transfer_characteristics = None;
                    let mut matrix_coefficients = None;

                    for (key, value) in color_config_object.iter() {
                        match key.as_ref() {
                            "bitDepth" => {
                                bit_depth =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "colorSpace" => {
                                color_space =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "colorPrimaries" => {
                                color_primaries =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "transferCharacteristics" => {
                                transfer_characteristics =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "matrixCoefficients" => {
                                matrix_coefficients =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            _ => {}
                        }
                    }

                    color_config = Some(MetadataColorInfoColorConfig {
                        bit_depth: bit_depth.ok_or(MetadataColorInfoError::MissingField("bitDepth"))?,
                        color_space: color_space.ok_or(MetadataColorInfoError::MissingField("colorSpace"))?,
                        color_primaries: color_primaries.ok_or(MetadataColorInfoError::MissingField("colorPrimaries"))?,
                        transfer_characteristics: transfer_characteristics
                            .ok_or(MetadataColorInfoError::MissingField("transferCharacteristics"))?,
                        matrix_coefficients: matrix_coefficients
                            .ok_or(MetadataColorInfoError::MissingField("matrixCoefficients"))?,
                    });
                }
                "hdrCll" => {
                    let Amf0Value::Object(hdr_cll_object) = value else {
                        return Err(MetadataColorInfoError::UnexpectedType {
                            expected: Amf0Marker::Object,
                            found: value.marker(),
                        });
                    };

                    let mut max_fall = None;
                    let mut max_cll = None;

                    for (key, value) in hdr_cll_object.iter() {
                        match key.as_ref() {
                            "maxFall" => {
                                max_fall =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "maxCll" => {
                                max_cll = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            _ => {}
                        }
                    }

                    hdr_cll = Some(MetadataColorInfoHdrCll {
                        max_fall: max_fall.ok_or(MetadataColorInfoError::MissingField("maxFall"))?,
                        max_cll: max_cll.ok_or(MetadataColorInfoError::MissingField("maxCll"))?,
                    });
                }
                "hdrMdcv" => {
                    let Amf0Value::Object(hdr_mdcv_object) = value else {
                        return Err(MetadataColorInfoError::UnexpectedType {
                            expected: Amf0Marker::Object,
                            found: value.marker(),
                        });
                    };

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
                                red_x = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            "redY" => {
                                red_y = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            "greenX" => {
                                green_x = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            "greenY" => {
                                green_y = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            "blueX" => {
                                blue_x = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            "blueY" => {
                                blue_y = Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                    expected: Amf0Marker::Number,
                                    found: value.marker(),
                                })?);
                            }
                            "whitePointX" => {
                                white_point_x =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "whitePointY" => {
                                white_point_y =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "maxLuminance" => {
                                max_luminance =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            "minLuminance" => {
                                min_luminance =
                                    Some(value.as_number().ok_or_else(|| MetadataColorInfoError::UnexpectedType {
                                        expected: Amf0Marker::Number,
                                        found: value.marker(),
                                    })?);
                            }
                            _ => {}
                        }
                    }

                    hdr_mdcv = Some(MetadataColorInfoHdrMdcv {
                        red_x: red_x.ok_or(MetadataColorInfoError::MissingField("redX"))?,
                        red_y: red_y.ok_or(MetadataColorInfoError::MissingField("redY"))?,
                        green_x: green_x.ok_or(MetadataColorInfoError::MissingField("greenX"))?,
                        green_y: green_y.ok_or(MetadataColorInfoError::MissingField("greenY"))?,
                        blue_x: blue_x.ok_or(MetadataColorInfoError::MissingField("blueX"))?,
                        blue_y: blue_y.ok_or(MetadataColorInfoError::MissingField("blueY"))?,
                        white_point_x: white_point_x.ok_or(MetadataColorInfoError::MissingField("whitePointX"))?,
                        white_point_y: white_point_y.ok_or(MetadataColorInfoError::MissingField("whitePointY"))?,
                        max_luminance: max_luminance.ok_or(MetadataColorInfoError::MissingField("maxLuminance"))?,
                        min_luminance: min_luminance.ok_or(MetadataColorInfoError::MissingField("minLuminance"))?,
                    });
                }
                _ => {}
            }
        }

        Ok(MetadataColorInfo {
            color_config: color_config.ok_or(MetadataColorInfoError::MissingField("colorConfig"))?,
            hdr_cll: hdr_cll.ok_or(MetadataColorInfoError::MissingField("hdrCll"))?,
            hdr_mdcv: hdr_mdcv.ok_or(MetadataColorInfoError::MissingField("hdrMdcv"))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoPacketMetadataEntry {
    ColorInfo(MetadataColorInfo),
    Other {
        key: String,
        object: Vec<(String, Amf0Value<'static>)>,
    },
}

impl VideoPacketMetadataEntry {
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
