use std::io;

use bytes::Bytes;
use isobmff::UnknownBox;
use isobmff::boxes::{
    ColourInformation, ColourInformationBox, NclxColourInformation, PixelAspectRatioBox, SampleEntry, SampleFlags,
    TrackRunBoxSample, VisualSampleEntry,
};
use scuffle_flv::video::header::VideoFrameType;
use scuffle_h265::boxes::{HEVCConfigurationBox, HEVCSampleEntryHev1};
use scuffle_h265::{HEVCDecoderConfigurationRecord, SpsRbsp};

use crate::TransmuxError;

pub(crate) fn stsd_entry(config: HEVCDecoderConfigurationRecord) -> Result<(HEVCSampleEntryHev1, SpsRbsp), TransmuxError> {
    let Some(sps) = config
        .arrays
        .iter()
        .find(|a| a.nal_unit_type == scuffle_h265::NALUnitType::SpsNut)
        .and_then(|v| v.nalus.first())
    else {
        return Err(TransmuxError::InvalidHEVCDecoderConfigurationRecord);
    };

    let sps = scuffle_h265::SpsNALUnit::parse(io::Cursor::new(sps.clone()))?.rbsp;

    let mut sub_boxes = vec![UnknownBox::try_from_box(PixelAspectRatioBox::default())?];

    if let Some(cc) = sps.vui_parameters.as_ref().map(|v| &v.video_signal_type) {
        let colr = ColourInformationBox {
            colour_info: ColourInformation::Nclx(NclxColourInformation {
                colour_primaries: cc.colour_primaries as u16,
                matrix_coefficients: cc.matrix_coeffs as u16,
                transfer_characteristics: cc.transfer_characteristics as u16,
                full_range_flag: cc.video_full_range_flag,
            }),
        };
        sub_boxes.push(UnknownBox::try_from_box(colr)?);
    }

    let visual_sample_entry = VisualSampleEntry::new(
        SampleEntry::default(),
        sps.cropped_width() as u16,
        sps.cropped_height() as u16,
        [0; 32],
    );

    let hev1 = HEVCSampleEntryHev1 {
        sample_entry: visual_sample_entry,
        config: HEVCConfigurationBox::new(config),
        mpeg4_extension: None,
        sub_boxes,
    };

    Ok((hev1, sps))
}

pub(crate) fn trun_sample(
    frame_type: VideoFrameType,
    composition_time: i32,
    duration: u32,
    data: &Bytes,
) -> Result<TrackRunBoxSample, TransmuxError> {
    Ok(TrackRunBoxSample {
        sample_duration: Some(duration),
        sample_composition_time_offset: Some(composition_time as i64),
        sample_flags: Some(SampleFlags {
            reserved: 0,
            is_leading: 0,
            sample_degradation_priority: 0,
            sample_depends_on: if frame_type == VideoFrameType::KeyFrame { 2 } else { 1 },
            sample_has_redundancy: 0,
            sample_is_depended_on: 0,
            sample_is_non_sync_sample: frame_type != VideoFrameType::KeyFrame,
            sample_padding_value: 0,
        }),
        sample_size: Some(data.len() as u32),
    })
}
