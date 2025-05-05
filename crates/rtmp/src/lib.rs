//! A crate for handling RTMP server connections.
//!
//! ## Specifications
//!
//! | Name | Version | Link | Comments |
//! | --- | --- | --- | --- |
//! | Adobe’s Real Time Messaging Protocol | `1.0` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/rtmp-v1-0-spec.pdf> | Refered to as 'Legacy RTMP spec' in this documentation |
//! | Enhancing RTMP, FLV | `v1-2024-02-29-r1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v1.pdf> | |
//! | Enhanced RTMP | `v2-2024-10-22-b1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v2.pdf> | Refered to as 'Enhanced RTMP spec' in this documentation |
//!
//! ## Example
//!
//! ```no_run
//! # use std::io::Cursor;
//! #
//! # use scuffle_rtmp::ServerSession;
//! # use scuffle_rtmp::session::server::{ServerSessionError, SessionData, SessionHandler};
//! # use tokio::net::TcpListener;
//! #
//! struct Handler;
//!
//! impl SessionHandler for Handler {
//!     async fn on_data(&mut self, stream_id: u32, data: SessionData) -> Result<(), ServerSessionError> {
//!         // Handle incoming video/audio/meta data
//!         Ok(())
//!     }
//!
//!     async fn on_publish(&mut self, stream_id: u32, app_name: &str, stream_name: &str) -> Result<(), ServerSessionError> {
//!         // Handle the publish event
//!         Ok(())
//!     }
//!
//!     async fn on_unpublish(&mut self, stream_id: u32) -> Result<(), ServerSessionError> {
//!         // Handle the unpublish event
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let listener = TcpListener::bind("[::]:1935").await.unwrap();
//!     // listening on [::]:1935
//!
//!     while let Ok((stream, addr)) = listener.accept().await {
//!         let session = ServerSession::new(stream, Handler);
//!
//!         tokio::spawn(async move {
//!             if let Err(err) = session.run().await {
//!                 // Handle the session error
//!             }
//!         });
//!     }
//! }
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
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

pub mod chunk;
pub mod command_messages;
pub mod error;
pub mod handshake;
pub mod messages;
pub mod protocol_control_messages;
pub mod session;
pub mod user_control_messages;

pub use session::server::ServerSession;

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use scuffle_future_ext::FutureExt;
    use tokio::process::Command;
    use tokio::sync::{mpsc, oneshot};

    use crate::session::server::{ServerSession, ServerSessionError, SessionData, SessionHandler};

    enum Event {
        Publish {
            stream_id: u32,
            app_name: String,
            stream_name: String,
            response: oneshot::Sender<Result<(), ServerSessionError>>,
        },
        Unpublish {
            stream_id: u32,
            response: oneshot::Sender<Result<(), ServerSessionError>>,
        },
        Data {
            stream_id: u32,
            data: SessionData,
            response: oneshot::Sender<Result<(), ServerSessionError>>,
        },
    }

    struct Handler(mpsc::Sender<Event>);

    impl SessionHandler for Handler {
        async fn on_publish(&mut self, stream_id: u32, app_name: &str, stream_name: &str) -> Result<(), ServerSessionError> {
            let (response, reciever) = oneshot::channel();

            self.0
                .send(Event::Publish {
                    stream_id,
                    app_name: app_name.to_string(),
                    stream_name: stream_name.to_string(),
                    response,
                })
                .await
                .unwrap();

            reciever.await.unwrap()
        }

        async fn on_unpublish(&mut self, stream_id: u32) -> Result<(), ServerSessionError> {
            let (response, reciever) = oneshot::channel();

            self.0.send(Event::Unpublish { stream_id, response }).await.unwrap();

            reciever.await.unwrap()
        }

        async fn on_data(&mut self, stream_id: u32, data: SessionData) -> Result<(), ServerSessionError> {
            let (response, reciever) = oneshot::channel();
            self.0
                .send(Event::Data {
                    stream_id,
                    data,
                    response,
                })
                .await
                .unwrap();

            reciever.await.unwrap()
        }
    }

    #[cfg(not(valgrind))] // test is time-sensitive, consider refactoring?
    #[tokio::test]
    async fn test_basic_rtmp_clean() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("failed to bind");
        let addr = listener.local_addr().unwrap();

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");

        let _ffmpeg = Command::new("ffmpeg")
            .args([
                "-loglevel",
                "debug",
                "-re",
                "-i",
                dir.join("avc_aac.mp4").to_str().expect("failed to get path"),
                "-r",
                "30",
                "-t",
                "1", // just for the test so it doesn't take too long
                "-c",
                "copy",
                "-f",
                "flv",
                &format!("rtmp://{}:{}/live/stream-key", addr.ip(), addr.port()),
            ])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("failed to execute ffmpeg");

        let (ffmpeg_stream, _) = listener
            .accept()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
            .expect("failed to accept");

        let (ffmpeg_handle, mut ffmpeg_event_reciever) = {
            let (ffmpeg_event_producer, ffmpeg_event_reciever) = mpsc::channel(1);
            let session = ServerSession::new(ffmpeg_stream, Handler(ffmpeg_event_producer));

            (
                tokio::spawn(async move {
                    let r = session.run().await;
                    println!("ffmpeg session ended: {r:?}");
                    r
                }),
                ffmpeg_event_reciever,
            )
        };

        let event = ffmpeg_event_reciever
            .recv()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
            .expect("failed to recv event");

        match event {
            Event::Publish {
                stream_id,
                app_name,
                stream_name,
                response,
            } => {
                assert_eq!(stream_id, 1);
                assert_eq!(app_name, "live");
                assert_eq!(stream_name, "stream-key");
                response.send(Ok(())).expect("failed to send response");
            }
            _ => panic!("unexpected event"),
        }

        let mut got_video = false;
        let mut got_audio = false;
        let mut got_metadata = false;

        while let Some(data) = ffmpeg_event_reciever
            .recv()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
        {
            match data {
                Event::Data {
                    stream_id,
                    response,
                    data,
                    ..
                } => {
                    match data {
                        SessionData::Video { .. } => got_video = true,
                        SessionData::Audio { .. } => got_audio = true,
                        SessionData::Amf0 { .. } => got_metadata = true,
                    }
                    response.send(Ok(())).expect("failed to send response");
                    assert_eq!(stream_id, 1);
                }
                Event::Unpublish { stream_id, response } => {
                    assert_eq!(stream_id, 1);
                    response.send(Ok(())).expect("failed to send response");
                    break;
                }
                _ => panic!("unexpected event"),
            }
        }

        assert!(got_video);
        assert!(got_audio);
        assert!(got_metadata);

        if ffmpeg_event_reciever
            .recv()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
            .is_some()
        {
            panic!("unexpected event");
        }

        assert!(
            ffmpeg_handle
                .await
                .expect("failed to join handle")
                .expect("failed to handle ffmpeg connection")
        );

        // TODO: Fix this assertion
        // assert!(ffmpeg.try_wait().expect("failed to wait for ffmpeg").is_none());
    }

    // test is time-sensitive, consider refactoring?
    // windows seems to not let us kill ffmpeg without it cleaning up the stream.
    #[cfg(all(not(valgrind), not(windows)))]
    #[tokio::test]
    async fn test_basic_rtmp_unclean() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("failed to bind");
        let addr = listener.local_addr().unwrap();

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");

        let mut ffmpeg = Command::new("ffmpeg")
            .args([
                "-loglevel",
                "debug",
                "-re",
                "-i",
                dir.join("avc_aac.mp4").to_str().expect("failed to get path"),
                "-r",
                "30",
                "-t",
                "1", // just for the test so it doesn't take too long
                "-c",
                "copy",
                "-f",
                "flv",
                &format!("rtmp://{}:{}/live/stream-key", addr.ip(), addr.port()),
            ])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("failed to execute ffmpeg");

        let (ffmpeg_stream, _) = listener
            .accept()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
            .expect("failed to accept");

        let (ffmpeg_handle, mut ffmpeg_event_reciever) = {
            let (ffmpeg_event_producer, ffmpeg_event_reciever) = mpsc::channel(1);
            let session = ServerSession::new(ffmpeg_stream, Handler(ffmpeg_event_producer));

            (
                tokio::spawn(async move {
                    let r = session.run().await;
                    println!("ffmpeg session ended: {r:?}");
                    r
                }),
                ffmpeg_event_reciever,
            )
        };

        let event = ffmpeg_event_reciever
            .recv()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
            .expect("failed to recv event");

        match event {
            Event::Publish {
                stream_id,
                app_name,
                stream_name,
                response,
            } => {
                assert_eq!(stream_id, 1);
                assert_eq!(app_name, "live");
                assert_eq!(stream_name, "stream-key");
                response.send(Ok(())).expect("failed to send response");
            }
            _ => panic!("unexpected event"),
        }

        let mut got_video = false;
        let mut got_audio = false;
        let mut got_metadata = false;

        while let Some(data) = ffmpeg_event_reciever
            .recv()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
        {
            match data {
                Event::Data {
                    stream_id,
                    response,
                    data,
                    ..
                } => {
                    assert_eq!(stream_id, 1);
                    match data {
                        SessionData::Video { .. } => got_video = true,
                        SessionData::Audio { .. } => got_audio = true,
                        SessionData::Amf0 { .. } => got_metadata = true,
                    }
                    response.send(Ok(())).expect("failed to send response");
                }
                _ => panic!("unexpected event"),
            }

            if got_video && got_audio && got_metadata {
                break;
            }
        }

        assert!(got_video);
        assert!(got_audio);
        assert!(got_metadata);

        ffmpeg.kill().await.expect("failed to kill ffmpeg");

        while let Some(data) = ffmpeg_event_reciever
            .recv()
            .with_timeout(Duration::from_millis(1000))
            .await
            .expect("timed out")
        {
            match data {
                Event::Data { response, .. } => {
                    response.send(Ok(())).expect("failed to send response");
                }
                _ => panic!("unexpected event"),
            }
        }

        // the server should have detected the ffmpeg process has died uncleanly
        assert!(
            !ffmpeg_handle
                .await
                .expect("failed to join handle")
                .expect("failed to handle ffmpeg connection")
        );
    }
}
