// It is not very clear if the onStatus message should be part of the NetConnection or NetStream commands.
// The legacy RTMP spec makes it look like it should be part of the NetStream commands while the enhanced-rtmp-v2 spec
// is very clear that it should be part of the NetConnection commands.
// In reality, it is used as a response message to both NetConnection and NetStream commands.
// This is why we have decided to put it in its own module.

mod define;
mod writer;

pub use define::*;
