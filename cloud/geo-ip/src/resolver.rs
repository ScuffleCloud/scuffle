use maxminddb::MaxMindDbError;

pub struct GeoIpResolver {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl GeoIpResolver {
    pub fn new_from_data(data: Vec<u8>) -> Result<Self, MaxMindDbError> {
        let reader = maxminddb::Reader::from_source(data)?;
        Ok(Self { reader })
    }

    pub fn lookup<'a, T: serde::Deserialize<'a>>(&'a self, ip: std::net::IpAddr) -> Result<Option<T>, MaxMindDbError> {
        self.reader.lookup::<T>(ip)
    }
}
