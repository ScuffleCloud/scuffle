use tinc::well_known::prost::Timestamp;

pub(crate) trait ProstTimestampExt {
    fn to_chrono(&self) -> chrono::DateTime<chrono::Utc>;
}

impl ProstTimestampExt for Timestamp {
    fn to_chrono(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32).unwrap_or_default()
    }
}
