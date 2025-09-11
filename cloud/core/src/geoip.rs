use std::path::PathBuf;

use anyhow::Context;

use crate::std_ext::ResultExt;

pub struct GeoIpResolver {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl GeoIpResolver {
    pub async fn new(path: &PathBuf) -> anyhow::Result<Self> {
        let data = tokio::fs::read(path).await.context("failed to open maxminddb file")?;
        let reader = maxminddb::Reader::from_source(data).context("failed to parse geoip file")?;

        Ok(Self { reader })
    }

    pub fn lookup<'a, T: serde::Deserialize<'a>>(&'a self, ip: std::net::IpAddr) -> Result<Option<T>, tonic::Status> {
        self.reader
            .lookup::<T>(ip)
            .into_tonic_internal_err("failed to lookup geoip info")
    }
}
