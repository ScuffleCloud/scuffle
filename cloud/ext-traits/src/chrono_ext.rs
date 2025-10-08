use prost_types::Timestamp;

pub trait ChronoDateTimeExt {
    fn to_prost_timestamp_utc(&self) -> Timestamp;
}

impl<Z: chrono::TimeZone> ChronoDateTimeExt for chrono::DateTime<Z> {
    fn to_prost_timestamp_utc(&self) -> Timestamp {
        let utc = self.to_utc();
        Timestamp {
            seconds: utc.timestamp(),
            nanos: utc.timestamp_subsec_nanos() as i32,
        }
    }
}
