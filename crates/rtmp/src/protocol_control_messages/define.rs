//! Protocol control messages as defined in 5.4.

/// 5.4.1. Set Chunk Size (1)
#[derive(Debug)]
pub struct ProtocolControlMessageSetChunkSize {
    pub chunk_size: u32,
}

// 5.4.2. Abort Message (2)
// 5.4.3. Acknowledgement (3)

/// 5.4.4. Window Acknowledgement Size (5)
#[derive(Debug)]
pub struct ProtocolControlMessageWindowAcknowledgementSize {
    pub acknowledgement_window_size: u32,
}

/// 5.4.5. Set Peer Bandwidth (6)
#[derive(Debug)]
pub struct ProtocolControlMessageSetPeerBandwidth {
    pub acknowledgement_window_size: u32,
    pub limit_type: ProtocolControlMessageSetPeerBandwidthLimitType,
}

/// 5.4.5. Set Peer Bandwidth (6) Limit Type
#[derive(Debug, PartialEq, Eq, Clone, Copy, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum ProtocolControlMessageSetPeerBandwidthLimitType {
    Hard = 0,
    Soft = 1,
    Dynamic = 2,
}
