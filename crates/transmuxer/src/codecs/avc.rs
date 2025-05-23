use bytes::Bytes;
use isobmff::UnknownBox;
use isobmff::boxes::{
    ColourInformation, ColourInformationBox, NclxColourInformation, PixelAspectRatioBox, SampleEntry, SampleFlags,
    TrackRunBoxSample, VisualSampleEntry,
};
use scuffle_flv::video::header::VideoFrameType;
use scuffle_h264::boxes::{AVCConfigurationBox, AVCSampleEntry1};
use scuffle_h264::{AVCDecoderConfigurationRecord, Sps};

use crate::TransmuxError;

pub(crate) fn stsd_entry<'a>(
    config: AVCDecoderConfigurationRecord<'a>,
    sps: &'a Sps,
) -> Result<AVCSampleEntry1<'a>, TransmuxError> {
    if config.sps.is_empty() {
        return Err(TransmuxError::InvalidAVCDecoderConfigurationRecord);
    }

    let mut sub_boxes = vec![UnknownBox::try_from_box(PixelAspectRatioBox::default())?];

    if let Some(cc) = sps.color_config.as_ref() {
        let colr = ColourInformationBox {
            colour_info: ColourInformation::Nclx(NclxColourInformation {
                colour_primaries: cc.color_primaries as u16,
                matrix_coefficients: cc.matrix_coefficients as u16,
                transfer_characteristics: cc.transfer_characteristics as u16,
                full_range_flag: cc.video_full_range_flag,
            }),
        };
        sub_boxes.push(UnknownBox::try_from_box(colr)?);
    }

    let visual_sample_entry =
        VisualSampleEntry::new(SampleEntry::default(), sps.width() as u16, sps.height() as u16, [0; 32]);

    Ok(AVCSampleEntry1 {
        visual_sample_entry,
        config: AVCConfigurationBox::new(config),
        mpeg4_extension: None,
        sub_boxes,
    })
}

pub(crate) fn trun_sample(
    frame_type: VideoFrameType,
    composition_time: u32,
    duration: u32,
    data: &Bytes,
) -> Result<TrackRunBoxSample, TransmuxError> {
    Ok(TrackRunBoxSample {
        sample_duration: Some(duration),
        sample_size: Some(data.len() as u32),
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
        sample_composition_time_offset: Some(composition_time as i64),
    })
}
