use std::io;

use bytes::Bytes;
use scuffle_flv::video::header::VideoFrameType;
use scuffle_h265::{HEVCDecoderConfigurationRecord, SpsRbsp};
use scuffle_mp4::DynBox;
use scuffle_mp4::types::colr::{ColorType, Colr};
use scuffle_mp4::types::hev1::Hev1;
use scuffle_mp4::types::hvcc::HvcC;
use scuffle_mp4::types::stsd::{SampleEntry, VisualSampleEntry};
use scuffle_mp4::types::trun::{TrunSample, TrunSampleFlag};

use crate::TransmuxError;

pub(crate) fn stsd_entry(config: HEVCDecoderConfigurationRecord) -> Result<(DynBox, SpsRbsp), TransmuxError> {
    let Some(sps) = config
        .arrays
        .iter()
        .find(|a| a.nal_unit_type == scuffle_h265::NALUnitType::SpsNut)
        .and_then(|v| v.nalus.first())
    else {
        return Err(TransmuxError::InvalidHEVCDecoderConfigurationRecord);
    };

    let sps = scuffle_h265::SpsNALUnit::parse(io::Cursor::new(sps.clone()))?.rbsp;

    let colr = sps.vui_parameters.as_ref().map(|v| &v.video_signal_type).map(|color_config| {
        Colr::new(ColorType::Nclx {
            color_primaries: color_config.colour_primaries as u16,
            matrix_coefficients: color_config.matrix_coeffs as u16,
            transfer_characteristics: color_config.transfer_characteristics as u16,
            full_range_flag: color_config.video_full_range_flag,
        })
    });

    Ok((
        Hev1::new(
            SampleEntry::new(VisualSampleEntry::new(
                sps.cropped_width() as u16,
                sps.cropped_height() as u16,
                colr,
            )),
            HvcC::new(config),
            None,
        )
        .into(),
        sps,
    ))
}

pub(crate) fn trun_sample(
    frame_type: VideoFrameType,
    composition_time: i32,
    duration: u32,
    data: &Bytes,
) -> Result<TrunSample, TransmuxError> {
    Ok(TrunSample {
        composition_time_offset: Some(composition_time as i64),
        duration: Some(duration),
        flags: Some(TrunSampleFlag {
            reserved: 0,
            is_leading: 0,
            sample_degradation_priority: 0,
            sample_depends_on: if frame_type == VideoFrameType::KeyFrame { 2 } else { 1 },
            sample_has_redundancy: 0,
            sample_is_depended_on: 0,
            sample_is_non_sync_sample: frame_type != VideoFrameType::KeyFrame,
            sample_padding_value: 0,
        }),
        size: Some(data.len() as u32),
    })
}
