//! Protocol control messages as defined in 5.4.

pub struct ProtocolControlMessageSetChunkSize(pub u32);

pub struct ProtocolControlMessageWindowAcknowledgementSize(pub u32);

pub struct ProtocolControlMessageSetPeerBandwidth {
    pub window_size: u32,
    pub limit_type: u8,
}
