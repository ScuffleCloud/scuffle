use std::sync::Arc;

use base64::Engine;

use crate::CoreConfig;

pub(crate) const ADMIN_DIRECTORY_API_USER_SCOPE: &str = "https://www.googleapis.com/auth/admin.directory.user.readonly";
const ALL_SCOPES: [&str; 4] = ["openid", "profile", "email", ADMIN_DIRECTORY_API_USER_SCOPE];
const REQUIRED_SCOPES: [&str; 3] = ["openid", "profile", "email"];

#[derive(serde_derive::Deserialize, Debug)]
pub(crate) struct GoogleToken {
    pub access_token: String,
    pub expires_in: u64,
    #[serde(deserialize_with = "deserialize_google_id_token")]
    pub id_token: GoogleIdToken,
    pub scope: String,
    pub token_type: String,
}

/// https://developers.google.com/identity/openid-connect/openid-connect#obtainuserinfo
#[derive(serde_derive::Deserialize, Debug, Clone)]
pub(crate) struct GoogleIdToken {
    pub sub: String,
    pub email: String,
    pub email_verified: bool,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub hd: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

fn deserialize_google_id_token<'de, D>(deserialzer: D) -> Result<GoogleIdToken, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let token: String = serde::Deserialize::deserialize(deserialzer)?;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(serde::de::Error::custom("Invalid ID token format"));
    }

    let payload = base64::prelude::BASE64_URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(serde::de::Error::custom)?;

    serde_json::from_slice(&payload).map_err(serde::de::Error::custom)
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum GoogleTokenError {
    #[error("invalid token type: {0}")]
    InvalidTokenType(String),
    #[error("missing scope: {0}")]
    MissingScope(String),
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
}

fn redirect_uri<G: CoreConfig>(global: &Arc<G>) -> String {
    global.dashboard_origin().join("/oauth2-callback/google").unwrap().to_string()
}

pub(crate) fn authorization_url<G: CoreConfig>(global: &Arc<G>, state: &str) -> String {
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={state}",
        global.google_oauth2_config().client_id,
        urlencoding::encode(&redirect_uri(global)),
        ALL_SCOPES.join("%20"), // URL-encoded space
    )
}

pub(crate) async fn request_tokens<G: CoreConfig>(global: &Arc<G>, code: &str) -> Result<GoogleToken, GoogleTokenError> {
    let tokens: GoogleToken = global
        .http_client()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", global.google_oauth2_config().client_id.as_str()),
            ("client_secret", global.google_oauth2_config().client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &redirect_uri(global)),
        ])
        .send()
        .await?
        .json()
        .await?;

    if tokens.token_type != "Bearer" {
        return Err(GoogleTokenError::InvalidTokenType(tokens.token_type));
    }

    if let Some(missing) = REQUIRED_SCOPES.iter().find(|scope| !tokens.scope.contains(*scope)) {
        return Err(GoogleTokenError::MissingScope(missing.to_string()));
    }

    Ok(tokens)
}

#[derive(serde_derive::Deserialize, Debug)]
pub(crate) struct GoogleWorkspaceUser {
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
    #[serde(rename = "customerId")]
    pub customer_id: String,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum GoogleWorkspaceGetUserError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("invalid status code: {0}")]
    InvalidStatusCode(reqwest::StatusCode),
}

pub(crate) async fn request_google_workspace_user<G: CoreConfig>(
    global: &Arc<G>,
    access_token: &str,
    user_id: &str,
) -> Result<Option<GoogleWorkspaceUser>, GoogleWorkspaceGetUserError> {
    let response = global
        .http_client()
        .get(format!("https://www.googleapis.com/admin/directory/v1/users/{user_id}"))
        .bearer_auth(access_token)
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(GoogleWorkspaceGetUserError::InvalidStatusCode(response.status()));
    }

    Ok(Some(response.json().await?))
}
