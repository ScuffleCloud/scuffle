use ffmpeg_sys_next::*;

use crate::error::FfmpegError;
use crate::smart_object::SmartPtr;
use crate::utils::check_i64;

pub struct Frame(SmartPtr<AVFrame>);

impl Clone for Frame {
    fn clone(&self) -> Self {
        unsafe { Self::wrap(av_frame_clone(self.0.as_ptr())).expect("failed to clone frame") }
    }
}

/// Safety: `Frame` is safe to send between threads.
unsafe impl Send for Frame {}

/// Safety: `Frame` is safe to share between threads.
unsafe impl Sync for Frame {}

#[derive(Clone)]
pub struct VideoFrame(pub Frame);

#[derive(Clone)]
pub struct AudioFrame(pub Frame);

impl Frame {
    pub fn new() -> Result<Self, FfmpegError> {
        // Safety: the pointer returned from av_frame_alloc is valid
        unsafe { Self::wrap(av_frame_alloc()) }
    }

    /// Safety: `ptr` must be a valid pointer to an `AVFrame`.
    unsafe fn wrap(ptr: *mut AVFrame) -> Result<Self, FfmpegError> {
        Ok(Self(
            // The caller guarantees that `ptr` is valid.
            SmartPtr::wrap_non_null(ptr, |ptr| av_frame_free(ptr)).ok_or(FfmpegError::Alloc)?,
        ))
    }

    pub fn alloc_frame_buffer(&mut self, alignment: Option<i32>) -> Result<(), FfmpegError> {
        // Safety: `self.as_mut_ptr()` is assumed to provide a valid mutable pointer to an
        // `AVFrame` structure. The `av_frame_get_buffer` function from FFMPEG allocates
        // and attaches a buffer to the `AVFrame` if it doesn't already exist.
        // It is the caller's responsibility to ensure that `self` is properly initialized
        // and represents a valid `AVFrame` instance.

        unsafe {
            let av_frame = self.as_mut_ptr();
            let ret = ffmpeg_sys_next::av_frame_get_buffer(av_frame, alignment.unwrap_or(0));
            if ret < 0 {
                return Err(FfmpegError::Code(ret.into()));
            }
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const AVFrame {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.0.as_mut_ptr()
    }

    pub fn video(self) -> VideoFrame {
        VideoFrame(self)
    }

    pub fn audio(self) -> AudioFrame {
        AudioFrame(self)
    }

    pub fn pts(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().pts)
    }

    pub fn set_pts(&mut self, pts: Option<i64>) {
        self.0.as_deref_mut_except().pts = pts.unwrap_or(AV_NOPTS_VALUE);
        self.0.as_deref_mut_except().best_effort_timestamp = pts.unwrap_or(AV_NOPTS_VALUE);
    }

    pub fn duration(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().duration)
    }

    pub fn set_duration(&mut self, duration: Option<i64>) {
        self.0.as_deref_mut_except().duration = duration.unwrap_or(AV_NOPTS_VALUE);
    }

    pub fn best_effort_timestamp(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().best_effort_timestamp)
    }

    pub fn dts(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().pkt_dts)
    }

    pub fn set_dts(&mut self, dts: Option<i64>) {
        self.0.as_deref_mut_except().pkt_dts = dts.unwrap_or(AV_NOPTS_VALUE);
    }

    pub fn time_base(&self) -> AVRational {
        self.0.as_deref_except().time_base
    }

    pub fn set_time_base(&mut self, time_base: AVRational) {
        self.0.as_deref_mut_except().time_base = time_base;
    }

    pub fn format(&self) -> i32 {
        self.0.as_deref_except().format
    }

    pub fn set_format(&mut self, format: i32) {
        self.0.as_deref_mut_except().format = format;
    }

    pub fn is_audio(&self) -> bool {
        self.0.as_deref_except().ch_layout.nb_channels != 0
    }

    pub fn is_video(&self) -> bool {
        self.0.as_deref_except().width != 0
    }

    pub fn linesize(&self, index: usize) -> Option<i32> {
        self.0.as_deref_except().linesize.get(index).copied()
    }
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("pts", &self.pts())
            .field("dts", &self.dts())
            .field("duration", &self.duration())
            .field("best_effort_timestamp", &self.best_effort_timestamp())
            .field("time_base", &self.time_base())
            .field("format", &self.format())
            .field("is_audio", &self.is_audio())
            .field("is_video", &self.is_video())
            .finish()
    }
}

impl VideoFrame {
    pub fn width(&self) -> usize {
        self.0 .0.as_deref_except().width as usize
    }

    pub fn height(&self) -> usize {
        self.0 .0.as_deref_except().height as usize
    }

    pub fn sample_aspect_ratio(&self) -> AVRational {
        self.0 .0.as_deref_except().sample_aspect_ratio
    }

    pub fn set_sample_aspect_ratio(&mut self, sample_aspect_ratio: AVRational) {
        self.0 .0.as_deref_mut_except().sample_aspect_ratio = sample_aspect_ratio;
    }

    pub fn set_width(&mut self, width: usize) {
        self.0 .0.as_deref_mut_except().width = width as i32;
    }

    pub fn set_height(&mut self, height: usize) {
        self.0 .0.as_deref_mut_except().height = height as i32;
    }

    pub fn is_keyframe(&self) -> bool {
        self.0 .0.as_deref_except().key_frame != 0
    }

    pub fn pict_type(&self) -> AVPictureType {
        self.0 .0.as_deref_except().pict_type
    }

    pub fn set_pict_type(&mut self, pict_type: AVPictureType) {
        self.0 .0.as_deref_mut_except().pict_type = pict_type;
    }

    pub fn data(&self, index: usize) -> Option<&[u8]> {
        unsafe {
            self.0
                 .0
                .as_deref_except()
                .data
                .get(index)
                .map(|ptr| std::slice::from_raw_parts(*ptr, self.linesize(index).unwrap() as usize * self.height()))
        }
    }
}

impl std::fmt::Debug for VideoFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFrame")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("sample_aspect_ratio", &self.sample_aspect_ratio())
            .field("pts", &self.pts())
            .field("dts", &self.dts())
            .field("duration", &self.duration())
            .field("best_effort_timestamp", &self.best_effort_timestamp())
            .field("time_base", &self.time_base())
            .field("format", &self.format())
            .field("is_audio", &self.is_audio())
            .field("is_video", &self.is_video())
            .field("is_keyframe", &self.is_keyframe())
            .finish()
    }
}

impl std::ops::Deref for VideoFrame {
    type Target = Frame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for VideoFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AudioFrame {
    // Sets channel layout to default with a channel count of `channel_count`.
    pub fn set_channel_layout_default(&mut self, channel_count: usize) -> Result<(), FfmpegError> {
        // Safety: `self.as_mut_ptr()` should return a valid mutable pointer to the internal
        // `AVFrame` structure. This block modifies the `ch_layout` field of the `AVFrame`
        // using FFMPEG's `av_channel_layout_default` function, which is expected to be safe
        // as long as the provided `channel_count` is valid and within acceptable ranges.
        unsafe {
            let av_frame = self.as_mut_ptr();
            ffmpeg_sys_next::av_channel_layout_default(&mut (*av_frame).ch_layout, channel_count as i32);

            // returns 0 if the channel layout is invalid
            if ffmpeg_sys_next::av_channel_layout_check(&(*av_frame).ch_layout) == 0 {
                return Err(FfmpegError::Arguments("Invalid default channel layout."));
            }
        }
        Ok(())
    }

    // Sets channel layout to a custom layout. Note that the channel count
    // is defined by the given `ffmpeg_sys_next::AVChannelLayout`.
    pub fn set_channel_layout_custom(&mut self, custom_layout: ffmpeg_sys_next::AVChannelLayout) -> Result<(), FfmpegError> {
        // Safety: `self.as_mut_ptr()` is assumed to return a valid mutable pointer to the internal
        // `AVFrame` structure. This block directly assigns a custom channel layout to the
        // `ch_layout` field. It is the caller's responsibility to ensure that the custom
        // layout is valid and properly initialized.
        unsafe {
            let av_frame = self.as_mut_ptr();
            (*av_frame).ch_layout = custom_layout;

            // returns 0 if the channel layout is invalid
            if ffmpeg_sys_next::av_channel_layout_check(&(*av_frame).ch_layout) == 0 {
                return Err(FfmpegError::Arguments("Invalid default channel layout."));
            }
        }
        Ok(())
    }

    pub fn channel_layout(&self) -> ffmpeg_sys_next::AVChannelLayout {
        self.0 .0.as_deref_except().ch_layout
    }

    pub fn channel_count(&self) -> usize {
        self.0 .0.as_deref_except().ch_layout.nb_channels as usize
    }

    pub fn nb_samples(&self) -> i32 {
        self.0 .0.as_deref_except().nb_samples
    }

    pub fn set_nb_samples(&mut self, nb_samples: usize) {
        self.0 .0.as_deref_mut_except().nb_samples = nb_samples as i32;
    }

    pub fn sample_rate(&self) -> i32 {
        self.0 .0.as_deref_except().sample_rate
    }

    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        self.0 .0.as_deref_mut_except().sample_rate = sample_rate as i32;
    }
}

impl std::fmt::Debug for AudioFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioFrame")
            .field("channel_count", &self.channel_count())
            .field("nb_samples", &self.nb_samples())
            .field("sample_rate", &self.sample_rate())
            .field("pts", &self.pts())
            .field("dts", &self.dts())
            .field("duration", &self.duration())
            .field("best_effort_timestamp", &self.best_effort_timestamp())
            .field("time_base", &self.time_base())
            .field("format", &self.format())
            .field("is_audio", &self.is_audio())
            .field("is_video", &self.is_video())
            .finish()
    }
}

impl std::ops::Deref for AudioFrame {
    type Target = Frame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AudioFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_YUV420P;
    use ffmpeg_sys_next::AVSampleFormat::AV_SAMPLE_FMT_S16;
    use ffmpeg_sys_next::{av_frame_get_buffer, AVRational, AV_PIX_FMT_RGB32};
    use insta::assert_debug_snapshot;
    use rand::{thread_rng, Rng};

    use crate::frame::Frame;

    #[test]
    fn test_frame_clone() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_PIX_FMT_YUV420P as i32);

        unsafe {
            let av_frame = frame.as_mut_ptr();
            (*av_frame).width = 16;
            (*av_frame).height = 16;

            assert!(av_frame_get_buffer(av_frame, 32) >= 0, "Failed to allocate buffer for frame.");

            let buf_size = (*av_frame).linesize[0] * (*av_frame).height;
            assert_eq!(buf_size, 512, "Allocated buffer size should match.");
        }

        frame.set_pts(Some(12));
        frame.set_dts(Some(34));
        frame.set_duration(Some(5));
        frame.set_time_base(AVRational { num: 1, den: 30 });
        frame.set_format(AV_PIX_FMT_YUV420P as i32);

        let cloned_frame = frame.clone();

        assert_eq!(
            format!("{:?}", frame),
            format!("{:?}", cloned_frame),
            "Cloned frame should be equal to the original frame."
        );
    }

    #[test]
    fn test_audio_conversion() {
        let frame = Frame::new().expect("Failed to create frame");
        let mut audio_frame = frame.audio();

        assert!(audio_frame.set_channel_layout_default(2).is_ok());
        assert!(audio_frame.is_audio(), "The frame should be identified as audio.");
        assert!(!audio_frame.is_video(), "The frame should not be identified as video.");
    }

    #[test]
    fn test_set_format() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_PIX_FMT_YUV420P as i32);
        assert_eq!(
            frame.format(),
            AV_PIX_FMT_YUV420P as i32,
            "The format should match the set value."
        );

        frame.set_format(AV_PIX_FMT_RGB32 as i32);
        assert_eq!(
            frame.format(),
            AV_PIX_FMT_RGB32 as i32,
            "The format should match the updated value."
        );
    }

    #[test]
    fn test_linesize() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_PIX_FMT_YUV420P as i32);
        let mut video_frame = frame.video();
        video_frame.set_width(1920);
        video_frame.set_height(1080);

        unsafe {
            let av_frame = video_frame.as_mut_ptr();
            assert!(av_frame_get_buffer(av_frame, 32) >= 0, "Failed to allocate buffer for frame.");
        }

        assert!(
            video_frame.linesize(0).unwrap_or(0) > 0,
            "Linesize should be greater than zero for valid index."
        );

        assert!(
            video_frame.linesize(100).is_none(),
            "Linesize at an invalid index should return None."
        );
    }

    #[test]
    fn test_frame_debug() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_pts(Some(12345));
        frame.set_dts(Some(67890));
        frame.set_duration(Some(1000));
        frame.set_time_base(AVRational { num: 1, den: 30 });
        frame.set_format(AV_PIX_FMT_YUV420P as i32);

        assert_debug_snapshot!(frame, @r"
        Frame {
            pts: Some(
                12345,
            ),
            dts: Some(
                67890,
            ),
            duration: Some(
                1000,
            ),
            best_effort_timestamp: Some(
                12345,
            ),
            time_base: AVRational {
                num: 1,
                den: 30,
            },
            format: 0,
            is_audio: false,
            is_video: false,
        }
        ");
    }

    #[test]
    fn test_sample_aspect_ratio() {
        let frame = Frame::new().expect("Failed to create frame");
        let mut video_frame = frame.video();
        let sample_aspect_ratio = AVRational { num: 16, den: 9 };
        video_frame.set_sample_aspect_ratio(sample_aspect_ratio);

        assert_eq!(
            video_frame.sample_aspect_ratio(),
            sample_aspect_ratio,
            "Sample aspect ratio should match the set value."
        );
    }

    #[test]
    fn test_pict_type() {
        use ffmpeg_sys_next::AVPictureType::AV_PICTURE_TYPE_I;
        let frame = Frame::new().expect("Failed to create frame");
        let mut video_frame = frame.video();
        video_frame.set_pict_type(AV_PICTURE_TYPE_I);

        assert_eq!(
            video_frame.pict_type(),
            AV_PICTURE_TYPE_I,
            "Picture type should match the set value."
        );
    }

    #[test]
    fn test_data_allocation_and_access() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_PIX_FMT_YUV420P as i32);
        let mut video_frame = frame.video();
        video_frame.set_width(16);
        video_frame.set_height(16);

        let randomized_data: Vec<u8>;
        unsafe {
            let av_frame = video_frame.as_mut_ptr();
            assert!(av_frame_get_buffer(av_frame, 32) >= 0, "Failed to allocate buffer for frame.");

            // randomize y-plane (data[0])
            let linesize = (*av_frame).linesize[0] as usize; // bytes per row
            let height = (*av_frame).height as usize; // total rows
            let data_ptr = (*av_frame).data[0]; // pointer to the Y-plane data

            if !data_ptr.is_null() {
                let data_slice = std::slice::from_raw_parts_mut(data_ptr, linesize * height);
                randomized_data = (0..data_slice.len())
                    .map(|_| thread_rng().gen()) // generate random data
                    .collect();
                data_slice.copy_from_slice(&randomized_data); // copy random data to the frame
            } else {
                panic!("Failed to get valid data pointer for Y-plane.");
            }
        }

        if let Some(data) = video_frame.data(0) {
            assert_eq!(data, randomized_data.as_slice(), "Data does not match randomized content.");
        } else {
            panic!("Data at index 0 should not be None.");
        }
    }

    #[test]
    fn test_video_frame_debug() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_pts(Some(12345));
        frame.set_dts(Some(67890));
        frame.set_duration(Some(1000));
        frame.set_time_base(AVRational { num: 1, den: 30 });
        frame.set_format(AV_PIX_FMT_YUV420P as i32);
        let mut video_frame = frame.video();
        video_frame.set_width(1920);
        video_frame.set_height(1080);
        video_frame.set_sample_aspect_ratio(AVRational { num: 16, den: 9 });

        assert_debug_snapshot!(video_frame, @r"
        VideoFrame {
            width: 1920,
            height: 1080,
            sample_aspect_ratio: AVRational {
                num: 16,
                den: 9,
            },
            pts: Some(
                12345,
            ),
            dts: Some(
                67890,
            ),
            duration: Some(
                1000,
            ),
            best_effort_timestamp: Some(
                12345,
            ),
            time_base: AVRational {
                num: 1,
                den: 30,
            },
            format: 0,
            is_audio: false,
            is_video: true,
            is_keyframe: false,
        }
        ");
    }

    #[test]
    fn test_set_channel_layout_default_invalid_count_error() {
        let frame = Frame::new().expect("Failed to create frame");
        let mut audio_frame = frame.audio();

        assert!(
            audio_frame.set_channel_layout_default(usize::MAX).is_err(),
            "Expected error for invalid channel count."
        );
    }

    #[test]
    fn test_set_channel_layout_custom_invalid_layout_error() {
        let frame = Frame::new().expect("Failed to create frame");
        let mut audio_frame = frame.audio();
        let custom_layout = ffmpeg_sys_next::AVChannelLayout {
            order: ffmpeg_sys_next::AVChannelOrder::AV_CHANNEL_ORDER_NATIVE,
            nb_channels: -1,
            u: ffmpeg_sys_next::AVChannelLayout__bindgen_ty_1 {
                mask: ffmpeg_sys_next::AV_CH_LAYOUT_STEREO,
            },
            opaque: std::ptr::null_mut(),
        };

        assert!(
            audio_frame.set_channel_layout_custom(custom_layout).is_err(),
            "Expected error for invalid custom channel layout"
        );
    }

    #[test]
    fn test_set_channel_layout_custom() {
        let frame = Frame::new().expect("Failed to create frame");
        let mut audio_frame = frame.audio();
        let custom_layout = ffmpeg_sys_next::AVChannelLayout {
            order: ffmpeg_sys_next::AVChannelOrder::AV_CHANNEL_ORDER_NATIVE,
            nb_channels: 2,
            u: ffmpeg_sys_next::AVChannelLayout__bindgen_ty_1 {
                mask: ffmpeg_sys_next::AV_CH_LAYOUT_STEREO,
            },
            opaque: std::ptr::null_mut(),
        };

        assert!(
            audio_frame.set_channel_layout_custom(custom_layout).is_ok(),
            "Failed to set custom channel layout"
        );

        let layout = audio_frame.channel_layout();
        assert_eq!(layout.nb_channels, 2, "Expected channel layout to have 2 channels (stereo).");
        assert_eq!(
            unsafe { layout.u.mask },
            ffmpeg_sys_next::AV_CH_LAYOUT_STEREO,
            "Expected channel mask to match AV_CH_LAYOUT_STEREO."
        );
        assert_eq!(
            layout.order,
            ffmpeg_sys_next::AVChannelOrder::AV_CHANNEL_ORDER_NATIVE,
            "Expected channel order to be AV_CHANNEL_ORDER_NATIVE."
        );
    }

    #[test]
    fn test_alloc_frame_buffer() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_S16 as i32);
        let mut audio_frame = frame.audio();
        audio_frame.set_nb_samples(1024);
        audio_frame.set_sample_rate(44100);

        assert!(
            audio_frame.set_channel_layout_default(2).is_ok(),
            "Failed to set default channel layout"
        );
        assert!(
            audio_frame.alloc_frame_buffer(None).is_ok(),
            "Failed to allocate buffer with no alignment (should default to 0)"
        );
        assert!(
            audio_frame.alloc_frame_buffer(Some(0)).is_ok(),
            "Failed to allocate buffer with None alignment"
        );
        assert!(
            audio_frame.alloc_frame_buffer(Some(32)).is_ok(),
            "Failed to allocate buffer with positive alignment"
        );
        assert!(
            audio_frame.alloc_frame_buffer(Some(-1)).is_ok(),
            "Failed to allocate buffer with negative alignment"
        );
    }

    #[test]
    fn test_alloc_frame_buffer_error() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_S16 as i32);
        let mut audio_frame = frame.audio();
        audio_frame.set_nb_samples(1024);

        assert!(
            audio_frame.alloc_frame_buffer(None).is_err(),
            "Should fail to allocate buffer with invalid frame and None alignment"
        );
        assert!(
            audio_frame.alloc_frame_buffer(Some(0)).is_err(),
            "Should fail to allocate buffer with invalid frame and 0 alignment"
        );
        assert!(
            audio_frame.alloc_frame_buffer(Some(32)).is_err(),
            "Should fail to allocate buffer with invalid frame and positive alignment"
        );
        assert!(
            audio_frame.alloc_frame_buffer(Some(-1)).is_err(),
            "Should fail to allocate buffer with invalid frame and negative alignment"
        );
    }

    #[test]
    fn test_nb_samples() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_S16 as i32);
        let mut audio_frame = frame.audio();
        audio_frame.set_nb_samples(1024);

        assert_eq!(
            audio_frame.nb_samples(),
            1024,
            "The number of samples should match the set value."
        );
    }

    #[test]
    fn test_sample_rate() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_S16 as i32);
        let mut audio_frame = frame.audio();
        audio_frame.set_sample_rate(44100);

        assert_eq!(
            audio_frame.sample_rate(),
            44100,
            "The sample rate should match the set value."
        );
    }

    #[test]
    fn test_audio_frame_debug() {
        let mut frame = Frame::new().expect("Failed to create frame");
        frame.set_format(AV_SAMPLE_FMT_S16 as i32);
        let mut audio_frame = frame.audio();
        audio_frame.set_nb_samples(1024);
        audio_frame.set_sample_rate(44100);
        audio_frame.set_pts(Some(12345));
        audio_frame.set_dts(Some(67890));
        audio_frame.set_duration(Some(512));
        audio_frame.set_time_base(AVRational { num: 1, den: 44100 });

        assert!(
            audio_frame.set_channel_layout_default(2).is_ok(),
            "Failed to set default channel layout"
        );
        assert_debug_snapshot!(audio_frame, @r"
        AudioFrame {
            channel_count: 2,
            nb_samples: 1024,
            sample_rate: 44100,
            pts: Some(
                12345,
            ),
            dts: Some(
                67890,
            ),
            duration: Some(
                512,
            ),
            best_effort_timestamp: Some(
                12345,
            ),
            time_base: AVRational {
                num: 1,
                den: 44100,
            },
            format: 1,
            is_audio: true,
            is_video: false,
        }
        ");
    }
}
