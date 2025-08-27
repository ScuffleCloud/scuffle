use tinc::well_known::prost::Timestamp;

pub(crate) trait ChronoDateTimeExt {
    fn to_prost_timestamp_utc(&self) -> Timestamp;
}

impl<Z: chrono::TimeZone> ChronoDateTimeExt for chrono::DateTime<Z> {
    fn to_prost_timestamp_utc(&self) -> Timestamp {
        Timestamp {
            seconds: self.to_utc().timestamp(),
            nanos: self.to_utc().timestamp_subsec_nanos() as i32,
        }
    }
}
