use std::io::{ErrorKind, Read};
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::std_ext::ResultExt;

#[derive(Debug, Clone, Serialize, Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct MaxMindConfig {
    /// The path to the MaxMindDB file
    #[default("./GeoLite2-City.mmdb".parse().unwrap())]
    pub path: PathBuf,
    /// If the file is not found, download it from maxmind
    pub download: Option<MaxMindDownloadConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, smart_default::SmartDefault)]
#[serde(default)]
pub struct MaxMindDownloadConfig {
    /// The MaxMindDB account ID
    pub account_id: String,
    /// The MaxMindDB license key
    pub license_key: String,
    #[default("https://download.maxmind.com/geoip/databases/GeoLite2-City/download?suffix=tar.gz".parse().unwrap())]
    pub url: Url,
    /// Cache the file to the path specified in `path`
    #[default(true)]
    pub cache_download: bool,
}

pub struct GeoIpResolver {
    reader: maxminddb::Reader<Vec<u8>>,
}

impl GeoIpResolver {
    pub async fn new(config: &MaxMindConfig) -> anyhow::Result<Self> {
        let file = tokio::fs::read(&config.path).await;
        let data = match (file, config.download.as_ref()) {
            (Ok(file), _) => file,
            (Err(e), Some(download_config)) if e.kind() == ErrorKind::NotFound => {
                tracing::warn!("geoip file not found on disk, downloading");

                let response = reqwest::Client::new()
                    .get(download_config.url.clone())
                    .basic_auth(&download_config.account_id, Some(&download_config.license_key))
                    .send()
                    .await
                    .context("failed to download geoip file")?;

                response.error_for_status_ref().context("failed to download geoip file")?;

                let gzip_tar = response.bytes().await.context("failed to download geoip file")?;

                let content = tokio::task::spawn_blocking(move || {
                    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(gzip_tar.as_ref()));
                    let mut buffer = Vec::new();

                    for entry in archive.entries().context("failed to extract geoip file")? {
                        let mut entry = entry.context("failed to extract geoip file")?;
                        let path = entry.path().context("failed to extract geoip file")?;
                        if path.extension().map(|e| e == "mmdb").unwrap_or(false) {
                            buffer.resize(entry.size() as usize, 0);
                            entry.read_exact(&mut buffer).context("failed to extract geoip file")?;
                            break;
                        }
                    }

                    anyhow::Ok(buffer)
                })
                .await
                .context("failed to extract geoip file")??;

                if download_config.cache_download {
                    tokio::fs::write(&config.path, &content)
                        .await
                        .context("failed to write geoip file")?;
                }

                content
            }
            (Err(e), _) => return Err(e.into()),
        };

        let reader = maxminddb::Reader::from_source(data).context("failed to parse geoip file")?;

        Ok(Self { reader })
    }

    pub fn lookup<'a, T: serde::Deserialize<'a>>(&'a self, ip: std::net::IpAddr) -> Result<Option<T>, tonic::Status> {
        self.reader
            .lookup::<T>(ip)
            .into_tonic_internal_err("failed to lookup geoip info")
    }
}
