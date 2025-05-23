use bytes::Bytes;
use isobmff::UnknownBox;
use isobmff::boxes::{
    ColourInformation, ColourInformationBox, NclxColourInformation, PixelAspectRatioBox, SampleEntry, SampleFlags,
    TrackRunBoxSample, VisualSampleEntry,
};
use scuffle_av1::boxes::{AV1CodecConfigurationBox, AV1SampleEntry};
use scuffle_av1::seq::SequenceHeaderObu;
use scuffle_av1::{AV1CodecConfigurationRecord, ObuHeader, ObuType};
use scuffle_bytes_util::zero_copy::ZeroCopyReader;
use scuffle_flv::video::header::VideoFrameType;

use crate::TransmuxError;

pub(crate) fn stsd_entry(config: AV1CodecConfigurationRecord) -> Result<(AV1SampleEntry, SequenceHeaderObu), TransmuxError> {
    let mut config_obu_reader = scuffle_bytes_util::zero_copy::Slice::from(config.config_obu.as_bytes());
    let header = ObuHeader::parse(&mut config_obu_reader.as_std())?;
    let data = if let Some(size) = header.size {
        config_obu_reader.try_read(size as usize)?
    } else {
        config_obu_reader.try_read_to_end()?
    };

    if header.obu_type != ObuType::SequenceHeader {
        return Err(TransmuxError::InvalidAv1DecoderConfigurationRecord);
    }

    let seq_obu = SequenceHeaderObu::parse(header, &mut std::io::Cursor::new(data))?;

    // Unfortunate there does not seem to be a way to get the
    // frame rate from the sequence header unless the timing_info is present
    // Which it almost never is.
    // So for AV1 we rely on the framerate being set in the scriptdata tag

    let colr = ColourInformationBox {
        colour_info: ColourInformation::Nclx(NclxColourInformation {
            colour_primaries: seq_obu.color_config.color_primaries as u16,
            matrix_coefficients: seq_obu.color_config.matrix_coefficients as u16,
            transfer_characteristics: seq_obu.color_config.transfer_characteristics as u16,
            full_range_flag: seq_obu.color_config.full_color_range,
        }),
    };

    let visual_sample_entry = VisualSampleEntry::new(
        SampleEntry::default(),
        seq_obu.max_frame_width as u16,
        seq_obu.max_frame_height as u16,
        [0; 32],
    );

    let av01 = AV1SampleEntry {
        sample_entry: visual_sample_entry,
        av1c: AV1CodecConfigurationBox::new(config),
        sub_boxes: vec![
            UnknownBox::try_from_box(PixelAspectRatioBox::default())?,
            UnknownBox::try_from_box(colr)?,
        ],
    };

    Ok((av01, seq_obu))
}

pub(crate) fn trun_sample(
    frame_type: VideoFrameType,
    duration: u32,
    data: &Bytes,
) -> Result<TrackRunBoxSample, TransmuxError> {
    Ok(TrackRunBoxSample {
        sample_composition_time_offset: None,
        sample_duration: Some(duration),
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
