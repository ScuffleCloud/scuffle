//! Protocol control messages as defined in 5.4.

#[derive(Debug)]
pub struct ProtocolControlMessageSetChunkSize {
    pub chunk_size: u32,
}

#[derive(Debug)]
pub struct ProtocolControlMessageWindowAcknowledgementSize {
    pub acknowledgement_window_size: u32,
}

#[derive(Debug)]
pub struct ProtocolControlMessageSetPeerBandwidth {
    pub acknowledgement_window_size: u32,
    pub limit_type: ProtocolControlMessageSetPeerBandwidthLimitType,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, num_derive::FromPrimitive)]
#[repr(u8)]
pub enum ProtocolControlMessageSetPeerBandwidthLimitType {
    Hard = 0,
    Soft = 1,
    Dynamic = 2,
}
