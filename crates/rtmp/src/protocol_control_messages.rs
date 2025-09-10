//! Protocol control messages as defined in 5.4.

pub mod reader;
pub mod writer;

/// Used to notify the peer of a new maximum chunk size.
///
/// Defined by:
/// - Legacy RTMP spec, 5.4.1. Set Chunk Size (1)
#[derive(Debug)]
pub struct ProtocolControlMessageSetChunkSize {
    /// > This field holds the new maximum chunk size,
    /// > in bytes, which will be used for all of the sender's subsequent
    /// > chunks until further notice. Valid sizes are `1` to `2147483647`
    /// > (`0x7FFFFFFF`) inclusive; however, all sizes greater than `16777215`
    /// > (`0xFFFFFF`) are equivalent since no chunk is larger than one
    /// > message, and no message is larger than `16777215` bytes.
    pub chunk_size: u32,
}

// Not implemented: 5.4.2. Abort Message (2)

/// Acknowledges the receipt of data.
///
/// Defined by:
/// - Legacy RTMP spec, 5.4.3. Acknowledgement (3)
#[derive(Debug)]
pub struct ProtocolControlMessageAcknowledgement {
    /// This field holds the number of bytes received so far.
    pub sequence_number: u32,
}

/// The client or the server sends this message to inform the peer of the
/// window size to use between sending acknowledgments.
///
/// Defined by:
/// - Legacy RTMP spec, 5.4.4. Window Acknowledgement Size (5)
#[derive(Debug)]
pub struct ProtocolControlMessageWindowAcknowledgementSize {
    /// The new window size to use.
    pub acknowledgement_window_size: u32,
}

/// > The client or the server sends this message to limit the output bandwidth of its peer.
/// > The peer receiving this message limits its
/// > output bandwidth by limiting the amount of sent but unacknowledged
/// > data to the window size indicated in this message.
///
/// Defined by:
/// - Legacy RTMP spec, 5.4.5. Set Peer Bandwidth (6)
#[derive(Debug)]
pub struct ProtocolControlMessageSetPeerBandwidth {
    /// The window size to limit the output bandwidth to.
    pub acknowledgement_window_size: u32,
    /// The limit type.
    pub limit_type: ProtocolControlMessageSetPeerBandwidthLimitType,
}

/// The limit type for [`ProtocolControlMessageSetPeerBandwidth`].
///
/// Defined by:
/// - Legacy RTMP spec, 5.4.5. Set Peer Bandwidth (6) Limit Type
#[derive(Debug, PartialEq, Eq, Clone, Copy, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum ProtocolControlMessageSetPeerBandwidthLimitType {
    /// > The peer SHOULD limit its output bandwidth to the indicated window size.
    Hard = 0,
    /// > The peer SHOULD limit its output bandwidth to the the
    /// > window indicated in this message or the limit already in effect,
    /// > whichever is smaller.
    Soft = 1,
    /// > If the previous Limit Type was Hard, treat this message
    /// > as though it was marked Hard, otherwise ignore this message.
    Dynamic = 2,
}
