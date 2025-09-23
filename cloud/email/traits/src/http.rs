pub trait HttpClientInterface: Send + Sync {
    fn external_http_client(&self) -> &reqwest::Client;
}
