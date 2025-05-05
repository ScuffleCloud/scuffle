use crate::AVPixelFormat;
use crate::error::{FfmpegError, FfmpegErrorCode};
use crate::ffi::*;
use crate::frame::VideoFrame;
use crate::smart_object::SmartPtr;

/// A scaler is a wrapper around an [`SwsContext`]. Which is used to scale or transform video frames.
pub struct VideoScaler {
    ptr: SmartPtr<SwsContext>,
    frame: VideoFrame,
    pixel_format: AVPixelFormat,
    width: i32,
    height: i32,
}

/// Safety: `Scaler` is safe to send between threads.
unsafe impl Send for VideoScaler {}

impl VideoScaler {
    /// Creates a new `Scaler` instance.
    pub fn new(
        input_width: i32,
        input_height: i32,
        incoming_pixel_fmt: AVPixelFormat,
        width: i32,
        height: i32,
        pixel_format: AVPixelFormat,
    ) -> Result<Self, FfmpegError> {
        // Safety: `sws_getContext` is safe to call, and the pointer returned is valid.
        let ptr = unsafe {
            sws_getContext(
                input_width,
                input_height,
                incoming_pixel_fmt.into(),
                width,
                height,
                pixel_format.into(),
                SWS_BILINEAR as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };

        let destructor = |ptr: &mut *mut SwsContext| {
            // Safety: `sws_freeContext` is safe to call.
            unsafe {
                sws_freeContext(*ptr);
            }

            *ptr = std::ptr::null_mut();
        };

        // Safety: `ptr` is a valid pointer & `destructor` has been setup to free the context.
        let ptr = unsafe { SmartPtr::wrap_non_null(ptr, destructor) }.ok_or(FfmpegError::Alloc)?;

        let frame = VideoFrame::builder()
            .width(width)
            .height(height)
            .pix_fmt(pixel_format)
            .build()?;

        Ok(Self {
            ptr,
            frame,
            pixel_format,
            width,
            height,
        })
    }

    /// Returns the pixel format of the scalar.
    pub const fn pixel_format(&self) -> AVPixelFormat {
        self.pixel_format
    }

    /// Returns the width of the scalar.
    pub const fn width(&self) -> i32 {
        self.width
    }

    /// Returns the height of the scalar.
    pub const fn height(&self) -> i32 {
        self.height
    }

    /// Processes a frame through the scalar.
    pub fn process<'a>(&'a mut self, frame: &VideoFrame) -> Result<&'a VideoFrame, FfmpegError> {
        // Safety: `frame` is a valid pointer, and `self.ptr` is a valid pointer.
        let frame_ptr = unsafe { frame.as_ptr().as_ref().unwrap() };
        // Safety: `self.frame` is a valid pointer.
        let self_frame_ptr = unsafe { self.frame.as_ptr().as_ref().unwrap() };

        // Safety: `sws_scale` is safe to call.
        FfmpegErrorCode(unsafe {
            sws_scale(
                self.ptr.as_mut_ptr(),
                frame_ptr.data.as_ptr() as *const *const u8,
                frame_ptr.linesize.as_ptr(),
                0,
                frame_ptr.height,
                self_frame_ptr.data.as_ptr(),
                self_frame_ptr.linesize.as_ptr(),
            )
        })
        .result()?;

        // Copy the other fields from the input frame to the output frame.
        self.frame.set_dts(frame.dts());
        self.frame.set_pts(frame.pts());
        self.frame.set_duration(frame.duration());
        self.frame.set_time_base(frame.time_base());

        Ok(&self.frame)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use insta::assert_debug_snapshot;
    use rand::Rng;

    use crate::frame::VideoFrame;
    use crate::scaler::{AVPixelFormat, VideoScaler};

    #[test]
    fn test_scalar_new() {
        let input_width = 1920;
        let input_height = 1080;
        let incoming_pixel_fmt = AVPixelFormat::Yuv420p;
        let output_width = 1280;
        let output_height = 720;
        let output_pixel_fmt = AVPixelFormat::Rgb24;
        let scalar = VideoScaler::new(
            input_width,
            input_height,
            incoming_pixel_fmt,
            output_width,
            output_height,
            output_pixel_fmt,
        );

        assert!(scalar.is_ok(), "Expected Scalar::new to succeed");
        let scalar = scalar.unwrap();

        assert_eq!(
            scalar.width(),
            output_width,
            "Expected Scalar width to match the output width"
        );
        assert_eq!(
            scalar.height(),
            output_height,
            "Expected Scalar height to match the output height"
        );
        assert_eq!(
            scalar.pixel_format(),
            output_pixel_fmt,
            "Expected Scalar pixel format to match the output pixel format"
        );
    }

    #[test]
    fn test_scalar_process() {
        let input_width = 1920;
        let input_height = 1080;
        let incoming_pixel_fmt = AVPixelFormat::Yuv420p;
        let output_width = 1280;
        let output_height = 720;
        let output_pixel_fmt = AVPixelFormat::Rgb24;

        let mut scalar = VideoScaler::new(
            input_width,
            input_height,
            incoming_pixel_fmt,
            output_width,
            output_height,
            output_pixel_fmt,
        )
        .expect("Failed to create Scalar");

        let mut input_frame = VideoFrame::builder()
            .width(input_width)
            .height(input_height)
            .pix_fmt(incoming_pixel_fmt)
            .build()
            .expect("Failed to create VideoFrame");

        // We need to fill the buffer with random data otherwise the result will be based off uninitialized data.
        let mut rng = rand::rng();

        for data_idx in 0..rusty_ffmpeg::ffi::AV_NUM_DATA_POINTERS {
            if let Some(mut data_buf) = input_frame.data_mut(data_idx as usize) {
                for row_idx in 0..data_buf.height() {
                    let row = data_buf.get_row_mut(row_idx as usize).unwrap();
                    rng.fill(row);
                }
            }
        }

        let result = scalar.process(&input_frame);

        assert!(
            result.is_ok(),
            "Expected Scalar::process to succeed, but got error: {result:?}"
        );

        let output_frame = result.unwrap();
        assert_debug_snapshot!(output_frame, @r"
        VideoFrame {
            width: 1280,
            height: 720,
            sample_aspect_ratio: Rational {
                numerator: 1,
                denominator: 1,
            },
            pts: None,
            dts: None,
            duration: Some(
                0,
            ),
            best_effort_timestamp: None,
            time_base: Rational {
                numerator: 0,
                denominator: 1,
            },
            format: AVPixelFormat::Rgb24,
            is_audio: false,
            is_video: true,
            is_keyframe: false,
        }
        ");
    }
}
