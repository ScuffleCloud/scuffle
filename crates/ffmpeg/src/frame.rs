use std::ops::{Index, IndexMut};
use std::ptr::NonNull;

use crate::consts::{Const, Mut};
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::rational::Rational;
use crate::smart_object::{SmartObject, SmartPtr};
use crate::utils::{check_i64, or_nopts};
use crate::{AVPictureType, AVPixelFormat, AVSampleFormat};

/// Wrapper around the data buffers of AVFrame that handles bottom-to-top line iteration
#[derive(Debug, PartialEq)]
pub struct FrameData {
    // this may point to the start of the last line of the buffer
    ptr: NonNull<u8>,
    linesize: i32,
    height: i32,
}

impl core::ops::Index<usize> for FrameData {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), index);
        }
        if self.linesize.is_positive() {
            // Safety: self.ptr + index is inside the bounds of the buffer
            let ptr = unsafe { self.ptr.byte_add(index) };
            // Safety: ptr is valid
            unsafe { ptr.as_ref() }
        } else {
            let stride = self.linesize.unsigned_abs() as usize;
            let line = index / stride;
            let line_pos = index % stride;
            // Safety: points to the start of the current line
            let current_line_ptr = unsafe { self.ptr.byte_sub(line * stride) };
            // Safety: points to the desired value within the current line
            let value_ptr = unsafe { current_line_ptr.byte_add(line_pos) };
            // Safety: value_ptr is valid
            unsafe { value_ptr.as_ref() }
        }
    }
}

impl core::ops::IndexMut<usize> for FrameData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len() {
            panic!("index out of bounds: the len is {} but the index is {}", self.len(), index);
        }
        if self.linesize.is_positive() {
            // Safety: self.ptr + index is inside the bounds of the buffer
            let mut ptr = unsafe { self.ptr.byte_add(index) };
            // Safety: ptr is valid
            unsafe { ptr.as_mut() }
        } else {
            let stride = self.linesize.unsigned_abs() as usize;
            let line = index / stride;
            let line_pos = index % stride;
            // Safety: points to the start of the current line
            let current_line_ptr = unsafe { self.ptr.byte_sub(line * stride) };
            // Safety: points to the desired value within the current line
            let mut value_ptr = unsafe { current_line_ptr.byte_add(line_pos) };
            // Safety: value_ptr is valid
            unsafe { value_ptr.as_mut() }
        }
    }
}

impl FrameData {
    /// Returns the height of the underlying data, in bytes
    pub const fn height(&self) -> i32 {
        self.height
    }

    /// Returns the linesize of the underlying data, in bytes. Negative if iteration
    /// order is bottom-to-top. [Reference](https://ffmpeg.org/doxygen/7.0/structAVFrame.html#aa52bfc6605f6a3059a0c3226cc0f6567)
    pub const fn linesize(&self) -> i32 {
        self.linesize
    }

    /// Returns the length of the underlying data, in bytes
    pub const fn len(&self) -> usize {
        (self.linesize.abs() * self.height) as usize
    }

    /// Returns true if the underlying data buffer is empty
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the byte at a given index
    pub fn get(&self, index: usize) -> Option<&u8> {
        if index < self.len() { Some(self.index(index)) } else { None }
    }

    /// Returns a mutable reference to the byte at a given index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        if index < self.len() {
            Some(self.index_mut(index))
        } else {
            None
        }
    }

    /// Returns a slice of row `index`, respecting bottom-to-top iteration order
    pub const fn get_row(&self, index: usize) -> Option<&[u8]> {
        if index >= self.height as usize {
            return None;
        }

        // Safety: this pointer is within bounds
        let start_ptr = unsafe { self.ptr.byte_offset(self.linesize as isize * index as isize) };
        // Safety: this slice is valid
        Some(unsafe { core::slice::from_raw_parts(start_ptr.as_ptr(), self.linesize.unsigned_abs() as usize) })
    }

    /// Returns a mutable slice of row `index`, respecting bottom-to-top iteration order
    pub const fn get_row_mut(&mut self, index: usize) -> Option<&mut [u8]> {
        if index >= self.height() as usize {
            return None;
        }

        // Safety: this pointer is within bounds
        let start_ptr = unsafe { self.ptr.byte_offset(self.linesize as isize * index as isize) };
        // Safety: this slice is valid
        Some(unsafe { core::slice::from_raw_parts_mut(start_ptr.as_ptr(), self.linesize.unsigned_abs() as usize) })
    }

    /// Fills the data buffer with `value`
    pub fn fill(&mut self, value: u8) {
        for row in 0..self.height() {
            let slice = self.get_row_mut(row as usize).expect("row is out of bounds");
            slice.fill(value);
        }
    }
}

/// A frame. Thin wrapper around [`AVFrame`].
pub struct GenericFrame(SmartPtr<AVFrame>);

impl Clone for GenericFrame {
    fn clone(&self) -> Self {
        // Safety: `av_frame_clone` is safe to call.
        let clone = unsafe { av_frame_clone(self.0.as_ptr()) };

        // Safety: The pointer here is valid.
        unsafe { Self::wrap(clone).expect("failed to clone frame") }
    }
}

/// Safety: `GenericFrame` is safe to send between threads.
unsafe impl Send for GenericFrame {}

/// Safety: `GenericFrame` is safe to share between threads.
unsafe impl Sync for GenericFrame {}

/// A video frame. Thin wrapper around [`GenericFrame`]. Like a frame but has specific video properties.
#[derive(Clone)]
pub struct VideoFrame(GenericFrame);

/// An audio frame. Thin wrapper around [`GenericFrame`]. Like a frame but has specific audio properties.
#[derive(Clone)]
pub struct AudioFrame(GenericFrame);

impl GenericFrame {
    /// Creates a new frame.
    pub(crate) fn new() -> Result<Self, FfmpegError> {
        // Safety: `av_frame_alloc` is safe to call.
        let frame = unsafe { av_frame_alloc() };

        // Safety: The pointer here is valid.
        unsafe { Self::wrap(frame).ok_or(FfmpegError::Alloc) }
    }

    /// Wraps a pointer to an `AVFrame`.
    /// Takes ownership of the frame, meaning it will be freed when the [`GenericFrame`] is dropped.
    ///
    /// # Safety
    /// `ptr` must be a valid pointer to an `AVFrame`.
    pub(crate) unsafe fn wrap(ptr: *mut AVFrame) -> Option<Self> {
        let destructor = |ptr: &mut *mut AVFrame| {
            // Safety: av_frame_free is safe to call & we own the pointer.
            unsafe { av_frame_free(ptr) }
        };

        // Safety: The safety comment of the function implies this is safe.
        unsafe { SmartPtr::wrap_non_null(ptr, destructor).map(Self) }
    }

    /// Allocates a buffer for the frame.
    ///
    /// # Safety
    /// This function is unsafe because the caller must ensure the frame has not been allocated yet.
    /// Also the frame must be properly initialized after the allocation as the data is not zeroed out.
    /// Therefore reading from the frame after allocation will result in reading uninitialized data.
    pub(crate) unsafe fn alloc_frame_buffer(&mut self, alignment: Option<i32>) -> Result<(), FfmpegError> {
        // Safety: `self.as_mut_ptr()` is assumed to provide a valid mutable pointer to an
        // `AVFrame` structure. The `av_frame_get_buffer` function from FFMPEG allocates
        // and attaches a buffer to the `AVFrame` if it doesn't already exist.
        // It is the caller's responsibility to ensure that `self` is properly initialized
        // and represents a valid `AVFrame` instance.
        FfmpegErrorCode(unsafe { av_frame_get_buffer(self.as_mut_ptr(), alignment.unwrap_or(0)) }).result()?;
        Ok(())
    }

    /// Returns a pointer to the frame.
    pub(crate) const fn as_ptr(&self) -> *const AVFrame {
        self.0.as_ptr()
    }

    /// Returns a mutable pointer to the frame.
    pub(crate) const fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.0.as_mut_ptr()
    }

    /// Make this frame a video frame.
    pub(crate) const fn video(self) -> VideoFrame {
        VideoFrame(self)
    }

    /// Make this frame an audio frame.
    pub(crate) const fn audio(self) -> AudioFrame {
        AudioFrame(self)
    }

    /// Returns the presentation timestamp of the frame, in `time_base` units.
    pub const fn pts(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().pts)
    }

    /// Sets the presentation timestamp of the frame, in `time_base` units.
    pub const fn set_pts(&mut self, pts: Option<i64>) {
        self.0.as_deref_mut_except().pts = or_nopts(pts);
        self.0.as_deref_mut_except().best_effort_timestamp = or_nopts(pts);
    }

    /// Returns the duration of the frame, in `time_base` units.
    pub const fn duration(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().duration)
    }

    /// Sets the duration of the frame, in `time_base` units.
    pub const fn set_duration(&mut self, duration: Option<i64>) {
        self.0.as_deref_mut_except().duration = or_nopts(duration);
    }

    /// Returns the best effort timestamp of the frame, in `time_base` units.
    pub const fn best_effort_timestamp(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().best_effort_timestamp)
    }

    /// Returns the decoding timestamp of the frame, in `time_base` units.
    pub const fn dts(&self) -> Option<i64> {
        check_i64(self.0.as_deref_except().pkt_dts)
    }

    /// Sets the decoding timestamp of the frame, in `time_base` units.
    pub(crate) const fn set_dts(&mut self, dts: Option<i64>) {
        self.0.as_deref_mut_except().pkt_dts = or_nopts(dts);
    }

    /// Returns the time base of the frame.
    pub fn time_base(&self) -> Rational {
        self.0.as_deref_except().time_base.into()
    }

    /// Sets the time base of the frame.
    pub fn set_time_base(&mut self, time_base: impl Into<Rational>) {
        self.0.as_deref_mut_except().time_base = time_base.into().into();
    }

    /// Returns the format of the frame.
    pub(crate) const fn format(&self) -> i32 {
        self.0.as_deref_except().format
    }

    /// Returns true if the frame is an audio frame.
    pub(crate) const fn is_audio(&self) -> bool {
        self.0.as_deref_except().ch_layout.nb_channels != 0
    }

    /// Returns true if the frame is a video frame.
    pub(crate) const fn is_video(&self) -> bool {
        self.0.as_deref_except().width != 0
    }

    /// Returns the linesize of the frame, in bytes.
    pub const fn linesize(&self, index: usize) -> Option<i32> {
        if index >= self.0.as_deref_except().linesize.len() {
            return None;
        }
        Some(self.0.as_deref_except().linesize[index])
    }
}

impl std::fmt::Debug for GenericFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericFrame")
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

#[bon::bon]
impl VideoFrame {
    /// Creates a new [`VideoFrame`]
    #[builder]
    pub fn new(
        width: i32,
        height: i32,
        pix_fmt: AVPixelFormat,
        #[builder(default = Rational::ONE)] sample_aspect_ratio: Rational,
        #[builder(default = AV_NOPTS_VALUE)] pts: i64,
        #[builder(default = AV_NOPTS_VALUE)] dts: i64,
        #[builder(default = 0)] duration: i64,
        #[builder(default = Rational::ZERO)] time_base: Rational,
        /// Alignment of the underlying data buffers, set to 0 for automatic.
        #[builder(default = 0)]
        alignment: i32,
    ) -> Result<Self, FfmpegError> {
        if width <= 0 || height <= 0 {
            return Err(FfmpegError::Arguments("width and height must be positive and not 0"));
        }
        if alignment < 0 {
            return Err(FfmpegError::Arguments("alignment must be positive"));
        }

        let mut generic = GenericFrame::new()?;
        let inner = generic.0.as_deref_mut_except();

        inner.pict_type = AVPictureType::None.0 as _;
        inner.width = width;
        inner.height = height;
        inner.format = pix_fmt.0;
        inner.pts = pts;
        inner.best_effort_timestamp = pts;
        inner.pkt_dts = dts;
        inner.duration = duration;
        inner.time_base = time_base.into();
        inner.sample_aspect_ratio = sample_aspect_ratio.into();

        // Safety: this is a brand new GenericFrame, with width, height and format set
        unsafe { generic.alloc_frame_buffer(Some(alignment))? };

        Ok(VideoFrame(generic))
    }

    /// Returns the width of the frame.
    pub const fn width(&self) -> usize {
        self.0.0.as_deref_except().width as usize
    }

    /// Returns the height of the frame.
    pub const fn height(&self) -> usize {
        self.0.0.as_deref_except().height as usize
    }

    /// Returns the sample aspect ratio of the frame.
    pub fn sample_aspect_ratio(&self) -> Rational {
        self.0.0.as_deref_except().sample_aspect_ratio.into()
    }

    /// Sets the sample aspect ratio of the frame.
    pub fn set_sample_aspect_ratio(&mut self, sample_aspect_ratio: impl Into<Rational>) {
        self.0.0.as_deref_mut_except().sample_aspect_ratio = sample_aspect_ratio.into().into();
    }

    /// Returns true if the frame is a keyframe.
    pub const fn is_keyframe(&self) -> bool {
        self.0.0.as_deref_except().key_frame != 0
    }

    /// Returns the picture type of the frame.
    pub const fn pict_type(&self) -> AVPictureType {
        AVPictureType(self.0.0.as_deref_except().pict_type as _)
    }

    /// Sets the picture type of the frame.
    pub const fn set_pict_type(&mut self, pict_type: AVPictureType) {
        self.0.0.as_deref_mut_except().pict_type = pict_type.0 as _;
    }

    /// Returns a reference to the data of the frame. By specifying the index of the plane.
    pub fn data(&self, index: usize) -> Option<Const<FrameData, '_>> {
        // Safety: av_pix_fmt_desc_get is safe to call
        let descriptor = unsafe { rusty_ffmpeg::ffi::av_pix_fmt_desc_get(self.format().into()) };
        // Safety: as_ref is safe to call here
        let descriptor = unsafe { descriptor.as_ref()? };

        let line = self.linesize(index)?;
        let height = {
            // palette data
            if descriptor.flags & rusty_ffmpeg::ffi::AV_PIX_FMT_FLAG_PAL as u64 != 0 && index == 1 {
                1
            } else if index > 0 {
                self.height() >> descriptor.log2_chroma_h
            } else {
                self.height()
            }
        };

        let raw = NonNull::new(*(self.0.0.as_deref_except().data.get(index)?))?;

        Some(Const::new(FrameData {
            ptr: raw,
            linesize: line,
            height: height as i32,
        }))
    }

    /// Returns a mutable reference to the data of the frame. By specifying the index of the plane.
    pub fn data_mut(&mut self, index: usize) -> Option<Mut<FrameData, '_>> {
        // Safety: av_pix_fmt_desc_get is safe to call
        let descriptor = unsafe { rusty_ffmpeg::ffi::av_pix_fmt_desc_get(self.format().into()) };
        // Safety: as_ref is safe to call here
        let descriptor = unsafe { descriptor.as_ref()? };

        let line = self.linesize(index)?;
        let height = {
            // palette data
            if descriptor.flags & rusty_ffmpeg::ffi::AV_PIX_FMT_FLAG_PAL as u64 != 0 && index == 1 {
                1
            } else if index > 0 {
                self.height() >> descriptor.log2_chroma_h
            } else {
                self.height()
            }
        };

        let raw = NonNull::new(*(self.0.0.as_deref_except().data.get(index)?))?;

        Some(Mut::new(FrameData {
            ptr: raw,
            linesize: line,
            height: height as i32,
        }))
    }

    /// Get the pixel format of the frame.
    pub const fn format(&self) -> AVPixelFormat {
        AVPixelFormat(self.0.0.as_deref_except().format)
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
    type Target = GenericFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for VideoFrame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A thin wrapper around `AVChannelLayout` to make it easier to use.
pub struct AudioChannelLayout(SmartObject<AVChannelLayout>);

impl Default for AudioChannelLayout {
    fn default() -> Self {
        // Safety: this is a c-struct and those are safe to zero out.
        let zeroed_layout = unsafe { std::mem::zeroed() };

        Self(SmartObject::new(zeroed_layout, Self::destructor))
    }
}

impl AudioChannelLayout {
    #[doc(hidden)]
    fn destructor(ptr: &mut AVChannelLayout) {
        // Safety: `av_channel_layout_uninit` is safe to call.
        unsafe { av_channel_layout_uninit(ptr) };
    }

    /// Creates a new `AudioChannelLayout` instance.
    pub fn new(channels: i32) -> Result<Self, FfmpegError> {
        let mut layout = Self::default();

        // Safety: `av_channel_layout_default` is safe to call.
        unsafe { av_channel_layout_default(layout.0.as_mut(), channels) };

        layout.validate()?;

        Ok(layout)
    }

    /// Copies this `AudioChannelLayout` instance.
    pub fn copy(&self) -> Result<Self, FfmpegError> {
        let mut new = Self::default();
        // Safety: av_channel_layout_copy is safe to call
        FfmpegErrorCode(unsafe { av_channel_layout_copy(new.0.inner_mut(), self.0.inner_ref()) }).result()?;
        Ok(new)
    }

    /// Returns a pointer to the channel layout.
    pub(crate) fn as_ptr(&self) -> *const AVChannelLayout {
        self.0.as_ref()
    }

    /// Validates the channel layout.
    pub fn validate(&self) -> Result<(), FfmpegError> {
        // Safety: `av_channel_layout_check` is safe to call
        if unsafe { av_channel_layout_check(self.0.as_ref()) } == 0 {
            return Err(FfmpegError::Arguments("invalid channel layout"));
        }

        Ok(())
    }

    /// Wraps an `AVChannelLayout` automatically calling `av_channel_layout_uninit` on drop.
    ///
    /// # Safety
    /// Requires that the layout can be safely deallocated with `av_channel_layout_uninit`
    pub unsafe fn wrap(layout: AVChannelLayout) -> Self {
        Self(SmartObject::new(layout, Self::destructor))
    }

    /// Returns the number of channels in the layout.
    pub fn channel_count(&self) -> i32 {
        self.0.as_ref().nb_channels
    }

    /// Consumes the `AudioChannelLayout` and returns the inner `AVChannelLayout`.
    /// The caller is responsible for calling `av_channel_layout_uninit` on the returned value.
    pub fn into_inner(self) -> AVChannelLayout {
        self.0.into_inner()
    }

    pub(crate) fn apply(mut self, layout: &mut AVChannelLayout) {
        std::mem::swap(layout, self.0.as_mut());
    }
}

#[bon::bon]
impl AudioFrame {
    /// Creates a new [`AudioFrame`]
    #[builder]
    pub fn new(
        channel_layout: AudioChannelLayout,
        nb_samples: i32,
        sample_fmt: AVSampleFormat,
        sample_rate: i32,
        #[builder(default = 0)] duration: i64,
        #[builder(default = AV_NOPTS_VALUE)] pts: i64,
        #[builder(default = AV_NOPTS_VALUE)] dts: i64,
        #[builder(default = Rational::ZERO)] time_base: Rational,
        /// Alignment of the underlying data buffers, set to 0 for automatic.
        #[builder(default = 0)]
        alignment: i32,
    ) -> Result<Self, FfmpegError> {
        if sample_rate <= 0 || nb_samples <= 0 {
            return Err(FfmpegError::Arguments(
                "sample_rate and nb_samples must be positive and not 0",
            ));
        }
        if alignment < 0 {
            return Err(FfmpegError::Arguments("alignment must be positive"));
        }

        let mut generic = GenericFrame::new()?;
        let inner = generic.0.as_deref_mut_except();

        channel_layout.apply(&mut inner.ch_layout);
        inner.nb_samples = nb_samples;
        inner.format = sample_fmt.into();
        inner.sample_rate = sample_rate;
        inner.duration = duration;
        inner.pts = pts;
        inner.best_effort_timestamp = pts;
        inner.time_base = time_base.into();
        inner.pkt_dts = dts;

        // Safety: this is a brand new GenericFrame, with nb_samples, ch_layout and format set
        unsafe { generic.alloc_frame_buffer(Some(alignment))? };

        Ok(Self(generic))
    }

    /// Returns the channel layout of the frame.
    pub fn channel_layout(&self) -> AudioChannelLayout {
        // Safety: the AudioFrame has already been initialized at this point, so
        // `av_channel_layout_uninit` is safe to call
        unsafe { AudioChannelLayout::wrap(self.0.0.as_deref_except().ch_layout) }
    }

    /// Returns the channel count of the frame.
    pub const fn channel_count(&self) -> usize {
        self.0.0.as_deref_except().ch_layout.nb_channels as usize
    }

    /// Returns the number of samples in the frame.
    pub const fn nb_samples(&self) -> i32 {
        self.0.0.as_deref_except().nb_samples
    }

    /// Returns the sample rate of the frame.
    pub const fn sample_rate(&self) -> i32 {
        self.0.0.as_deref_except().sample_rate
    }

    /// Sets the sample rate of the frame.
    pub const fn set_sample_rate(&mut self, sample_rate: usize) {
        self.0.0.as_deref_mut_except().sample_rate = sample_rate as i32;
    }

    /// Returns a reference to the data of the frame. By specifying the index of the plane.
    pub fn data(&self, index: usize) -> Option<&[u8]> {
        let ptr = *self.0.0.as_deref_except().data.get(index)?;

        if ptr.is_null() {
            return None;
        }

        // this is the length of the buffer ptr points to, in bytes
        let linesize = self.linesize(index)?;

        if linesize.is_negative() {
            return None;
        }

        // Safety: ptr is not null and linesize is the correct length for the slice type
        Some(unsafe { core::slice::from_raw_parts(ptr, linesize as usize) })
    }

    /// Returns a mutable reference to the data of the frame. By specifying the index of the plane.
    pub fn data_mut(&mut self, index: usize) -> Option<&mut [u8]> {
        let ptr = *self.0.0.as_deref_except().data.get(index)?;

        if ptr.is_null() {
            return None;
        }

        // this is the length of the buffer ptr points to, in bytes
        let linesize = self.linesize(index)?;

        if linesize.is_negative() {
            return None;
        }

        // Safety: ptr is not null and linesize is the correct length for the slice type
        Some(unsafe { core::slice::from_raw_parts_mut(ptr, linesize as usize) })
    }

    /// Get the sample format of the frame.
    pub const fn format(&self) -> AVSampleFormat {
        AVSampleFormat(self.0.0.as_deref_except().format)
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
    type Target = GenericFrame;

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
    use insta::assert_debug_snapshot;
    use rand::{Rng, rng};

    use super::FrameData;
    use crate::frame::{AudioChannelLayout, AudioFrame, GenericFrame, VideoFrame};
    use crate::rational::Rational;
    use crate::{AVChannelOrder, AVPictureType, AVPixelFormat, AVSampleFormat};

    #[test]
    fn test_frame_clone() {
        let frame = VideoFrame::builder()
            .width(16)
            .height(16)
            .pts(12)
            .dts(34)
            .duration(5)
            .time_base(Rational::static_new::<1, 30>())
            .pix_fmt(AVPixelFormat::Yuv420p)
            .build()
            .expect("failed to build VideoFrame");

        let cloned_frame = frame.clone();

        assert_eq!(
            format!("{frame:?}"),
            format!("{:?}", cloned_frame),
            "Cloned frame should be equal to the original frame."
        );
    }

    #[test]
    fn test_audio_conversion() {
        let mut frame = GenericFrame::new().expect("Failed to create frame");
        AudioChannelLayout::new(2)
            .unwrap()
            .apply(&mut frame.0.as_deref_mut_except().ch_layout);
        let audio_frame = frame.audio();

        assert!(audio_frame.is_audio(), "The frame should be identified as audio.");
        assert!(!audio_frame.is_video(), "The frame should not be identified as video.");
    }

    #[test]
    fn test_linesize() {
        let frame = VideoFrame::builder()
            .width(1920)
            .height(1080)
            .pix_fmt(AVPixelFormat::Yuv420p)
            .build()
            .expect("Failed to create frame");

        assert!(
            frame.linesize(0).unwrap_or(0) > 0,
            "Linesize should be greater than zero for valid index."
        );

        assert!(
            frame.linesize(100).is_none(),
            "Linesize at an invalid index should return None."
        );
    }

    #[test]
    fn test_frame_debug() {
        let mut frame = GenericFrame::new().expect("Failed to create frame");
        frame.set_pts(Some(12345));
        frame.set_dts(Some(67890));
        frame.set_duration(Some(1000));
        frame.set_time_base(Rational::static_new::<1, 30>());
        frame.0.as_deref_mut_except().format = AVPixelFormat::Yuv420p.into();

        assert_debug_snapshot!(frame, @r"
        GenericFrame {
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
            time_base: Rational {
                numerator: 1,
                denominator: 30,
            },
            format: 0,
            is_audio: false,
            is_video: false,
        }
        ");
    }

    #[test]
    fn test_sample_aspect_ratio() {
        let frame = GenericFrame::new().expect("Failed to create frame");
        let mut video_frame = frame.video();
        let sample_aspect_ratio = Rational::static_new::<16, 9>();
        video_frame.set_sample_aspect_ratio(sample_aspect_ratio);

        assert_eq!(
            video_frame.sample_aspect_ratio(),
            sample_aspect_ratio,
            "Sample aspect ratio should match the set value."
        );
    }

    #[test]
    fn test_pict_type() {
        let frame = GenericFrame::new().expect("Failed to create frame");
        let mut video_frame = frame.video();
        video_frame.set_pict_type(AVPictureType::Intra);

        assert_eq!(
            video_frame.pict_type(),
            AVPictureType::Intra,
            "Picture type should match the set value."
        );
    }

    #[test]
    fn test_data_allocation_and_access() {
        let mut video_frame = VideoFrame::builder()
            .width(16)
            .height(16)
            .pix_fmt(AVPixelFormat::Yuv420p)
            .alignment(32)
            .build()
            .expect("Failed to create VideoFrame");

        let mut randomized_data: Vec<Vec<u8>> = Vec::with_capacity(video_frame.height());

        if let Some(mut data) = video_frame.data_mut(0) {
            for row in 0..data.height() {
                let data_slice = data.get_row_mut(row as usize).unwrap();
                randomized_data.push(
                    (0..data_slice.len())
                        .map(|_| rng().random::<u8>()) // generate random data
                        .collect(),
                );
                data_slice.copy_from_slice(&randomized_data[row as usize]); // copy random data to the frame
            }
        } else {
            panic!("Failed to get valid data buffer for Y-plane.");
        }

        if let Some(data) = video_frame.data(0) {
            for row in 0..data.height() {
                let data_slice = data.get_row(row as usize).unwrap();
                assert_eq!(
                    data_slice,
                    randomized_data[row as usize].as_slice(),
                    "Data does not match randomized content."
                );
            }
        } else {
            panic!("Data at index 0 should not be None.");
        }
    }

    #[test]
    fn test_video_frame_debug() {
        let video_frame = VideoFrame::builder()
            .pts(12345)
            .dts(67890)
            .duration(1000)
            .time_base(Rational::static_new::<1, 30>())
            .pix_fmt(AVPixelFormat::Yuv420p)
            .width(1920)
            .height(1080)
            .sample_aspect_ratio(Rational::static_new::<16, 9>())
            .build()
            .expect("Failed to create a new VideoFrame");

        assert_debug_snapshot!(video_frame, @r"
        VideoFrame {
            width: 1920,
            height: 1080,
            sample_aspect_ratio: Rational {
                numerator: 16,
                denominator: 9,
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
            time_base: Rational {
                numerator: 1,
                denominator: 30,
            },
            format: AVPixelFormat::Yuv420p,
            is_audio: false,
            is_video: true,
            is_keyframe: false,
        }
        ");
    }

    #[test]
    fn test_set_channel_layout_custom_invalid_layout_error() {
        // Safety: This is safe to be deallocated by the layout destructor.
        let custom_layout = unsafe {
            AudioChannelLayout::wrap(crate::ffi::AVChannelLayout {
                order: AVChannelOrder::Native.into(),
                nb_channels: -1,
                u: crate::ffi::AVChannelLayout__bindgen_ty_1 { mask: 2 },
                opaque: std::ptr::null_mut(),
            })
        };
        let audio_frame = AudioFrame::builder()
            .channel_layout(custom_layout)
            .nb_samples(123)
            .sample_fmt(AVSampleFormat::S16)
            .sample_rate(44100)
            .build();

        assert!(audio_frame.is_err(), "Expected error for invalid custom channel layout");
    }

    #[test]
    fn test_set_channel_layout_custom() {
        // Safety: This is safe to be deallocated by the layout destructor.
        let custom_layout = unsafe {
            AudioChannelLayout::wrap(crate::ffi::AVChannelLayout {
                order: AVChannelOrder::Native.into(),
                nb_channels: 2,
                u: crate::ffi::AVChannelLayout__bindgen_ty_1 { mask: 3 },
                opaque: std::ptr::null_mut(),
            })
        };

        let audio_frame = AudioFrame::builder()
            .channel_layout(custom_layout)
            .nb_samples(123)
            .sample_fmt(AVSampleFormat::S16)
            .sample_rate(44100)
            .build()
            .expect("Failed to create AudioFrame with custom layout");

        let layout = audio_frame.channel_layout();
        assert_eq!(
            layout.channel_count(),
            2,
            "Expected channel layout to have 2 channels (stereo)."
        );
        assert_eq!(
            // Safety: this should be a mask not a pointer.
            unsafe { layout.0.u.mask },
            3,
            "Expected channel mask to match AV_CH_LAYOUT_STEREO."
        );
        assert_eq!(
            AVChannelOrder(layout.0.order as _),
            AVChannelOrder::Native,
            "Expected channel order to be AV_CHANNEL_ORDER_NATIVE."
        );
    }

    #[test]
    fn test_alloc_frame_buffer() {
        let cases = [(0, true), (3, true), (32, true), (-1, false)];

        for alignment in cases {
            let frame = AudioFrame::builder()
                .sample_fmt(AVSampleFormat::S16)
                .nb_samples(1024)
                .channel_layout(AudioChannelLayout::new(1).expect("failed to create a new AudioChannelLayout"))
                .alignment(alignment.0)
                .sample_rate(44100)
                .build();

            assert_eq!(frame.is_ok(), alignment.1)
        }
    }

    #[test]
    fn test_alloc_frame_buffer_error() {
        let cases = [None, Some(0), Some(32), Some(-1)];

        for alignment in cases {
            let mut frame = GenericFrame::new().expect("Failed to create frame");
            // Safety: frame is not yet allocated
            frame.0.as_deref_mut_except().format = AVSampleFormat::S16.into();
            frame.0.as_deref_mut_except().nb_samples = 1024;

            assert!(
                // Safety: `frame` is a valid pointer. And we dont attempt to read from the frame until after the allocation.
                unsafe { frame.alloc_frame_buffer(alignment).is_err() },
                "Should fail to allocate buffer with invalid frame and alignment {alignment:?}"
            );
        }
    }

    #[test]
    fn test_sample_rate() {
        let mut audio_frame = AudioFrame::builder()
            .channel_layout(AudioChannelLayout::new(2).expect("Failed to create a new AudioChannelLayout"))
            .nb_samples(123)
            .sample_fmt(AVSampleFormat::S16)
            .sample_rate(44100)
            .build()
            .expect("Failed to create AudioFrame with custom layout");

        audio_frame.set_sample_rate(48000);

        assert_eq!(
            audio_frame.sample_rate(),
            48000,
            "The sample rate should match the set value."
        );
    }

    #[test]
    fn test_audio_frame_debug() {
        let audio_frame = AudioFrame::builder()
            .sample_fmt(AVSampleFormat::S16)
            .channel_layout(AudioChannelLayout::new(2).expect("failed to create a new AudioChannelLayout"))
            .nb_samples(1024)
            .sample_rate(44100)
            .pts(12345)
            .dts(67890)
            .duration(512)
            .time_base(Rational::static_new::<1, 44100>())
            .build()
            .expect("failed to create a new AudioFrame");

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
            time_base: Rational {
                numerator: 1,
                denominator: 44100,
            },
            format: AVSampleFormat::S16,
            is_audio: true,
            is_video: false,
        }
        ");
    }

    #[test]
    fn frame_data_read() {
        let data: &mut [u8] = &mut [1, 2, 3, 4, 5, 6];

        let frame_data = FrameData {
            ptr: core::ptr::NonNull::new(data.as_mut_ptr()).unwrap(),
            linesize: 3,
            height: 2,
        };

        assert_eq!(frame_data[0], 1);
        assert_eq!(frame_data[5], 6);

        assert_eq!(frame_data.get_row(0).unwrap(), [1, 2, 3]);
        assert_eq!(frame_data.get_row(1).unwrap(), [4, 5, 6]);
        assert!(frame_data.get_row(2).is_none());
    }

    #[test]
    fn frame_data_read_inverse() {
        let data: &mut [u8] = &mut [1, 2, 3, 4, 5, 6];
        let linesize: i32 = -3;
        let height: i32 = 2;
        // Safety: this is a valid pointer
        let end_ptr = unsafe { data.as_mut_ptr().byte_offset(((height - 1) * linesize.abs()) as isize) };

        let frame_data = FrameData {
            ptr: core::ptr::NonNull::new(end_ptr).unwrap(),
            linesize,
            height,
        };

        assert_eq!(frame_data[0], 4);
        assert_eq!(frame_data[3], 1);
        assert_eq!(frame_data[5], 3);

        assert_eq!(frame_data.get_row(0).unwrap(), [4, 5, 6]);
        assert_eq!(frame_data.get_row(1).unwrap(), [1, 2, 3]);
        assert!(frame_data.get_row(2).is_none());
    }

    #[test]
    fn frame_data_read_out_of_bounds() {
        let data: &mut [u8] = &mut [1, 2, 3, 4, 5, 6];

        let linesize: i32 = -3;
        let height: i32 = 2;
        // Safety: this is a valid pointer
        let end_ptr = unsafe { data.as_mut_ptr().byte_offset(((height - 1) * linesize.abs()) as isize) };

        let inverse_frame_data = FrameData {
            ptr: core::ptr::NonNull::new(end_ptr).unwrap(),
            linesize,
            height,
        };

        let frame_data = FrameData {
            ptr: core::ptr::NonNull::new(data.as_mut_ptr()).unwrap(),
            linesize: linesize.abs(),
            height,
        };

        assert!(
            std::panic::catch_unwind(|| {
                let _ = inverse_frame_data[6];
            })
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| {
                let _ = frame_data[6];
            })
            .is_err()
        );
    }

    #[test]
    fn frame_data_write() {
        let data: &mut [u8] = &mut [1, 2, 3, 4, 5, 6];

        let mut frame_data = FrameData {
            ptr: core::ptr::NonNull::new(data.as_mut_ptr()).unwrap(),
            linesize: 3,
            height: 2,
        };

        for i in 1..frame_data.len() {
            frame_data[i] = frame_data[0]
        }

        for i in 0..frame_data.len() {
            assert_eq!(frame_data[i], 1, "all bytes of frame_data should be 0")
        }
    }
}
