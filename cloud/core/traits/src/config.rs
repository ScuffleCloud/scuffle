use std::borrow::Cow;

pub trait ConfigInterface: Send + Sync {
    fn service_bind(&self) -> std::net::SocketAddr;
    fn swagger_ui_enabled(&self) -> bool;
    fn turnstile_secret_key(&self) -> &str;
    fn email_from_name(&self) -> &str;
    fn email_from_address(&self) -> &str;
    fn dashboard_origin(&self) -> Option<&url::Url>;
    fn timeout_config(&self) -> TimeoutConfig;
    fn google_oauth2_config(&self) -> GoogleOAuth2Config<'_>;
}

pub struct TimeoutConfig {
    pub max_request_diff: std::time::Duration,
    pub user_session: std::time::Duration,
    pub user_session_token: std::time::Duration,
    pub mfa: std::time::Duration,
    pub new_user_email_request: std::time::Duration,
    pub user_session_request: std::time::Duration,
    pub magic_link_request: std::time::Duration,
}

pub struct GoogleOAuth2Config<'a> {
    pub client_id: Cow<'a, str>,
    pub client_secret: Cow<'a, str>,
}
