nutype_enum::nutype_enum! {
    pub enum EventType(u16) {
        StreamBegin = 0,
        StreamEOF = 1,
        StreamDry = 2,
        SetBufferLength = 3,
        StreamIsRecorded = 4,
        PingRequest = 6,
        PingResponse = 7,
    }
}

pub struct EventMessageStreamBegin {
    pub stream_id: u32,
}
