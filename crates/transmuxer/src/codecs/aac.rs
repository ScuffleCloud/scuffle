use bytes::Bytes;
use fixed::FixedU32;
use isobmff::boxes::{AudioSampleEntry, SampleEntry, SampleFlags, TrackRunBoxSample};
use scuffle_aac::PartialAudioSpecificConfig;
use scuffle_bytes_util::zero_copy::U24Be;
use scuffle_flv::audio::header::legacy::{SoundSize, SoundType};
use scuffle_mp4::boxes::{ESDBox, MP4AudioSampleEntry};
use scuffle_mp4::object_description::{
    DecoderConfigDescriptor, DescriptorTag, ESDescriptor, ObjectTypeIndication, StreamType, UnknownDescriptor,
};

use crate::TransmuxError;

pub(crate) fn stsd_entry<'a>(
    sound_size: SoundSize,
    sound_type: SoundType,
    data: Bytes,
) -> Result<(MP4AudioSampleEntry<'a>, PartialAudioSpecificConfig), TransmuxError> {
    let aac_config = scuffle_aac::PartialAudioSpecificConfig::parse(&data)?;

    let channelcount = match sound_type {
        SoundType::Mono => 1,
        SoundType::Stereo => 2,
        _ => return Err(TransmuxError::InvalidAudioChannels),
    };

    let samplesize = match sound_size {
        SoundSize::Bit8 => 8,
        SoundSize::Bit16 => 16,
        _ => return Err(TransmuxError::InvalidAudioSampleSize),
    };

    let es = ESDescriptor {
        es_id: 2,
        stream_priority: 0,
        depends_on_es_id: Some(0),
        url_string: None,
        ocr_es_id: Some(0),
        dec_config_descr: DecoderConfigDescriptor {
            object_type_indication: ObjectTypeIndication::Audio14496_3, // AAC
            stream_type: StreamType::AudioStream,
            up_stream: false,
            reserved: true,
            buffer_size_db: U24Be(0),
            max_bitrate: 0,
            avg_bitrate: 0,
            dec_specific_info: Some(UnknownDescriptor::new(DescriptorTag::DecSpecificInfoTag, data.into())),
            profile_level_indication_index_descr: vec![],
            unknown_descriptors: vec![],
        },
        sl_config_descr: None,
        unknown_descriptors: vec![],
    };
    let mp4a = MP4AudioSampleEntry {
        sample_entry: AudioSampleEntry::new(
            SampleEntry::default(),
            channelcount,
            samplesize,
            FixedU32::from_bits(aac_config.sampling_frequency),
        ),
        es: ESDBox::new(es),
    };

    Ok((mp4a, aac_config))
}

pub(crate) fn trun_sample(data: &Bytes) -> Result<(TrackRunBoxSample, u32), TransmuxError> {
    Ok((
        TrackRunBoxSample {
            sample_duration: Some(1024),
            sample_composition_time_offset: None,
            sample_flags: Some(SampleFlags {
                reserved: 0,
                is_leading: 0,
                sample_degradation_priority: 0,
                sample_depends_on: 2,
                sample_has_redundancy: 0,
                sample_is_depended_on: 0,
                sample_is_non_sync_sample: false,
                sample_padding_value: 0,
            }),
            sample_size: Some(data.len() as u32),
        },
        1024,
    ))
}
