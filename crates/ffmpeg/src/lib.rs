//! A crate designed to provide a simple interface to the native ffmpeg c-bindings.
//!
//! ## Why do we need this?
//!
//! This crate aims to provide a simple-safe interface to the native ffmpeg c-bindings.
//!
//! Currently this crate only supports the latest versions of ffmpeg (7.x.x).
//!
//! ## How is this different from other ffmpeg crates?
//!
//! The other main ffmpeg crate is [ffmpeg-next](https://github.com/zmwangx/rust-ffmpeg).
//!
//! This crate adds a few features and has a safer API. Notably it adds the ability to provide an in-memory decode / encode buffer.
//!
//! ## Examples
//!
//! ### Decoding a audio/video file
//!
//! ```rust
//! # use std::path::PathBuf;
//! # use scuffle_ffmpeg::AVMediaType;
//! # fn test_fn() -> Result<(), Box<dyn std::error::Error>> {
//! # let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets").join("avc_aac.mp4");
//! // 1. Store the input of the file from the path `path`
//! // this can be any seekable io stream (std::io::Read + std::io::Seek)
//! // if you don't have seek, you can just use Input::new(std::io::Read) (no seeking support)
//! let mut input = scuffle_ffmpeg::io::Input::seekable(std::fs::File::open(path)?)?;
//! // 2. Get the streams from the input
//! let streams = input.streams();
//!
//! dbg!(&streams);
//!
//! // 3. Store video and audio stream into respective their variables; we will panic if either one doesn't exist.
//! let best_video_stream = streams.best(AVMediaType::Video).expect("no video stream found");
//! let best_audio_stream = streams.best(AVMediaType::Audio).expect("no audio stream found");
//!
//! dbg!(&best_video_stream);
//! dbg!(&best_audio_stream);
//!
//! // 4. Create and store the respective video and audio decoders; we will panic if either one doesn't exist, or is an invalid decoder.
//! let mut video_decoder = scuffle_ffmpeg::decoder::Decoder::new(&best_video_stream)?.video().expect("not an video decoder");
//! let mut audio_decoder = scuffle_ffmpeg::decoder::Decoder::new(&best_audio_stream)?.audio().expect("not an audio decoder");
//!
//! dbg!(&video_decoder);
//! dbg!(&audio_decoder);
//!
//! // 5. Get the stream index of the video and audio streams.
//! let video_stream_index = best_video_stream.index();
//! let audio_stream_index = best_audio_stream.index();
//!
//! // 6. Iterate over the packets in the input.
//! for packet in input.packets() {
//!     let packet = packet?;
//!     // 7. Send the packet to the respective decoder.
//!     // 8. Receive the frame from the decoder.
//!     if packet.stream_index() == video_stream_index {
//!         video_decoder.send_packet(&packet)?;
//!         while let Some(frame) = video_decoder.receive_frame()? {
//!             dbg!(&frame);
//!         }
//!     } else if packet.stream_index() == audio_stream_index {
//!         audio_decoder.send_packet(&packet)?;
//!         while let Some(frame) = audio_decoder.receive_frame()? {
//!             dbg!(&frame);
//!         }
//!     }
//! }
//!
//! // 9. Send the EOF to the decoders.
//! video_decoder.send_eof()?;
//! audio_decoder.send_eof()?;
//!
//! // 10. Receive the remaining frames from the decoders.
//! while let Some(frame) = video_decoder.receive_frame()? {
//!     dbg!(&frame);
//! }
//!
//! while let Some(frame) = audio_decoder.receive_frame()? {
//!     dbg!(&frame);
//! }
//! # Ok(())
//! # }
//! # test_fn().expect("failed to run test");
//! ```
//!
//! ### Re-encoding a audio/video file
//!
//! ```rust
//! # use std::path::PathBuf;
//! # use scuffle_ffmpeg::{AVMediaType, AVCodecID};
//! # use scuffle_ffmpeg::encoder::{AudioEncoderSettings, VideoEncoderSettings};
//! # use scuffle_ffmpeg::io::OutputOptions;
//! # use scuffle_ffmpeg::frame::AudioChannelLayout;
//! #
//! # fn test_fn() -> Result<(), Box<dyn std::error::Error>> {
//! # let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets").join("avc_aac.mp4");
//! // 1. Create an input for reading. In this case we open it from a std::fs::File, however
//! // it can be from any seekable io stream (std::io::Read + std::io::Seek) for example a std::io::Cursor.
//! // It can also be a non-seekable stream in that case you can use Input::new(std::io::Read)
//! let input = scuffle_ffmpeg::io::Input::seekable(std::fs::File::open(path)?)?;
//!
//! // 2. Get the streams from the input.
//! let streams = input.streams();
//!
//! // 3. Store the best video and audio streams into respective their variables; we will panic if either one doesn't exist.
//! let best_video_stream = streams.best(AVMediaType::Video).expect("no video stream found");
//! let best_audio_stream = streams.best(AVMediaType::Audio).expect("no audio stream found");
//!
//! // 4. Create and store the respective video and audio decoders; we will panic if either one doesn't exist, or is an invalid decoder.
//! let mut video_decoder = scuffle_ffmpeg::decoder::Decoder::new(&best_video_stream)?.video().expect("not an video decoder");
//! let mut audio_decoder = scuffle_ffmpeg::decoder::Decoder::new(&best_audio_stream)?.audio().expect("not an audio decoder");
//!
//! // 5. Create an output for writing. In this case we use a std::io::Cursor, however it can be any seekable io stream (std::io::Read + std::io::Seek) for example a std::io::Cursor.
//! // It can also be a non-seekable stream in that case you can use Output::new(std::io::Read)
//! let mut output = scuffle_ffmpeg::io::Output::seekable(std::io::Cursor::new(Vec::new()), OutputOptions::builder().format_name("mp4")?.build())?;
//!
//! // 6. Find the respective encoders for the video and audio streams.
//! let x264 = scuffle_ffmpeg::codec::EncoderCodec::by_name("libx264").expect("no h264 encoder found");
//! let aac = scuffle_ffmpeg::codec::EncoderCodec::new(AVCodecID::Aac).expect("no aac encoder found");
//!
//! // 7. Create the respective encoder settings for the video and audio streams.
//! let video_settings = VideoEncoderSettings::builder()
//!     .width(video_decoder.width())
//!     .height(video_decoder.height())
//!     .frame_rate(video_decoder.frame_rate())
//!     .pixel_format(video_decoder.pixel_format())
//!     .build();
//!
//! let audio_settings = AudioEncoderSettings::builder()
//!     .sample_rate(audio_decoder.sample_rate())
//!     .ch_layout(AudioChannelLayout::new(audio_decoder.channels()).expect("invalid channel layout"))
//!     .sample_fmt(audio_decoder.sample_format())
//!     .build();
//!
//! // 8. Create the respective encoders for the video and audio streams.
//! let mut video_encoder = scuffle_ffmpeg::encoder::Encoder::new(x264, &mut output, best_video_stream.time_base(), best_video_stream.time_base(), video_settings).expect("not an video encoder");
//! let mut audio_encoder = scuffle_ffmpeg::encoder::Encoder::new(aac, &mut output, best_audio_stream.time_base(), best_audio_stream.time_base(), audio_settings).expect("not an audio encoder");
//!
//! // 9. Write the header to the output.
//! output.write_header()?;
//!
//! loop {
//!     let mut audio_done = false;
//!     let mut video_done = false;
//!
//!     // 10. Receive the frame from the decoders.
//!     // 11. Send the frame to the encoders.
//!     // 12. Receive the packet from the encoders.
//!     // 13. Write the packet to the output.
//!
//!     if let Some(frame) = audio_decoder.receive_frame()? {
//!         audio_encoder.send_frame(&frame)?;
//!         while let Some(packet) = audio_encoder.receive_packet()? {
//!             output.write_packet(&packet)?;
//!         }
//!     } else {
//!         audio_done = true;
//!     }
//!
//!     if let Some(frame) = video_decoder.receive_frame()? {
//!         video_encoder.send_frame(&frame)?;
//!         while let Some(packet) = video_encoder.receive_packet()? {
//!             output.write_packet(&packet)?;
//!         }
//!     } else {
//!         video_done = true;
//!     }
//!
//!     // 14. Break the loop if both the audio and video are done.
//!     if audio_done && video_done {
//!         break;
//!     }
//! }
//!
//! // 15. Send the EOF to the decoders.
//! video_decoder.send_eof()?;
//! audio_decoder.send_eof()?;
//!
//! // 16. Receive the remaining packets from the encoders.
//! while let Some(packet) = video_encoder.receive_packet()? {
//!     output.write_packet(&packet)?;
//! }
//!
//! while let Some(packet) = audio_encoder.receive_packet()? {
//!     output.write_packet(&packet)?;
//! }
//!
//! // 17. Write the trailer to the output.
//! output.write_trailer()?;
//!
//! // 18. Do something with the output data (write to disk, upload to s3, etc).
//! let output_data = output.into_inner();
//! # drop(output_data);
//! # Ok(())
//! # }
//! # test_fn().expect("failed to run test");
//! ```
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]

/// Codec specific functionality.
pub mod codec;
/// Constants.
pub mod consts;
/// Decoder specific functionality.
pub mod decoder;
/// Dictionary specific functionality.
pub mod dict;
/// Encoder specific functionality.
pub mod encoder;
/// Error handling.
pub mod error;
/// Filter graph specific functionality.
pub mod filter_graph;
/// Frame specific functionality.
pub mod frame;
/// Input/Output specific functionality.
pub mod io;
/// Logging specific functionality.
pub mod log;
/// Packet specific functionality.
pub mod packet;
/// Rational number specific functionality.
pub mod rational;
/// [`frame::AudioFrame`] resampling and format conversion.
pub mod resampler;
/// Scaler specific functionality.
pub mod scaler;
/// Stream specific functionality.
pub mod stream;
/// Utility functionality.
pub mod utils;

pub use rusty_ffmpeg::ffi;

mod smart_object;

mod enums;

pub use enums::*;
