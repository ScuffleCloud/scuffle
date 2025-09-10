//! NetStream command messages.

use scuffle_amf0::{Amf0Object, Amf0Value};
use scuffle_bytes_util::StringCow;
use serde_derive::{Deserialize, Serialize};

pub mod reader;

/// NetStream commands as defined in 7.2.2.
#[derive(Debug, Clone, PartialEq)]
pub enum NetStreamCommand<'a> {
    /// Play command.
    Play {
        /// All values in the command.
        ///
        /// See the legacy RTMP spec for details.
        values: Vec<Amf0Value<'static>>,
    },
    /// Play2 command.
    Play2 {
        /// All values in the command.
        ///
        /// See the legacy RTMP spec for details.
        parameters: Amf0Object<'static>,
    },
    /// Delete stream command.
    DeleteStream {
        /// ID of the stream to delete.
        stream_id: f64,
    },
    /// Close stream command.
    CloseStream,
    /// Receive audio command.
    ReceiveAudio {
        /// true or false to indicate whether to receive audio or not.
        receive_audio: bool,
    },
    /// Receive video command.
    ReceiveVideo {
        /// true or false to indicate whether to receive video or not.
        receive_video: bool,
    },
    /// Publish command.
    Publish {
        /// Name with which the stream is published.
        publishing_name: StringCow<'a>,
        /// Type of publishing.
        publishing_type: NetStreamCommandPublishPublishingType<'a>,
    },
    /// Seek command.
    Seek {
        /// Number of milliseconds to seek into the playlist.
        milliseconds: f64,
    },
    /// Pause command.
    Pause {
        /// true or false, to indicate pausing or resuming play.
        pause: bool,
        /// Number of milliseconds at which the
        /// the stream is paused or play resumed.
        /// This is the current stream time at the
        /// Client when stream was paused. When the
        /// playback is resumed, the server will
        /// only send messages with timestamps
        /// greater than this value.
        milliseconds: f64,
    },
}

/// Type of publishing.
///
/// Appears as part of the [`NetStreamCommand::Publish`] command.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NetStreamCommandPublishPublishingType<'a> {
    /// Citing the legacy RTMP spec, page 46:
    /// Live data is published without recording it in a file.
    Live,
    /// Citing the legacy RTMP spec, page 46:
    /// > The stream is published and the
    /// > data is recorded to a new file. The file
    /// > is stored on the server in a
    /// > subdirectory within the directory that
    /// > contains the server application. If the
    /// > file already exists, it is overwritten.
    Record,
    /// Citing the legacy RTMP spec, page 46:
    /// The stream is published and the
    /// data is appended to a file. If no file
    /// is found, it is created.
    Append,
    /// Any other value.
    #[serde(untagged, borrow)]
    Unknown(StringCow<'a>),
}
