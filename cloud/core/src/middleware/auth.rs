use std::str::FromStr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use diesel::{BoolExpressionMethods, ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use hmac::Mac;
use sha2::Digest;

use crate::CoreConfig;
use crate::http_ext::RequestExt;
use crate::middleware::IpAddressInfo;
use crate::models::{UserSession, UserSessionTokenId};
use crate::schema::user_sessions;

const TOKEN_ID_HEADER: HeaderName = HeaderName::from_static("scuf-token-id");
const TIMESTAMP_HEADER: HeaderName = HeaderName::from_static("scuf-timestamp");
const NONCE_HEADER: HeaderName = HeaderName::from_static("scuf-nonce");
const BODY_SHA256_HEADER: HeaderName = HeaderName::from_static("scuf-body-sha256");

const AUTHENTICATION_METHOD_HEADER: HeaderName = HeaderName::from_static("scuf-auth-method");
const AUTHENTICATION_HMAC_HEADER: HeaderName = HeaderName::from_static("scuf-auth-hmac");

pub(crate) async fn auth<G: CoreConfig>(req: Request, next: Next) -> Result<Response, StatusCode> {
    let (mut parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, 1024 * 1024 * 1024) // 1GiB
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;

    let global = parts
        .extensions
        .global::<G>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ip_info = parts
        .extensions
        .ip_address_info()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(session) = get_and_update_active_session(&global, &ip_info, &parts, &bytes).await? {
        parts.extensions.insert(session);
    }

    Ok(next.run(Request::from_parts(parts, Body::from(bytes))).await)
}

fn get_auth_header<'a, T>(headers: &'a HeaderMap, header_name: &HeaderName) -> Result<Option<T>, StatusCode>
where
    T: FromStr + 'a,
{
    match headers.get(header_name) {
        Some(h) => {
            let s = h.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            Ok(Some(s.parse().map_err(|_| StatusCode::BAD_REQUEST)?))
        }
        None => Ok(None),
    }
}

#[derive(Debug, thiserror::Error)]
enum AuthenticationMethodParseError {
    #[error("unknown authentication algorithm")]
    UnknownAlgorithm,
    #[error("invalid header format")]
    InvalidHeaderFormat,
}

#[derive(Debug)]
enum AuthenticationAlgorithm {
    HmacSha256,
}

impl FromStr for AuthenticationAlgorithm {
    type Err = AuthenticationMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HMAC-SHA256" => Ok(AuthenticationAlgorithm::HmacSha256),
            _ => Err(AuthenticationMethodParseError::UnknownAlgorithm),
        }
    }
}

#[derive(Debug)]
struct AuthenticationMethod {
    pub algorithm: AuthenticationAlgorithm,
    pub headers: Vec<HeaderName>,
}

impl FromStr for AuthenticationMethod {
    type Err = AuthenticationMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ';').collect();
        if parts.len() != 2 {
            return Err(AuthenticationMethodParseError::InvalidHeaderFormat);
        }

        let algorithm: AuthenticationAlgorithm = parts[0].parse()?;
        let headers: Vec<HeaderName> = parts[1]
            .split(',')
            .map(|h| HeaderName::from_str(h.trim()).map_err(|_| AuthenticationMethodParseError::InvalidHeaderFormat))
            .collect::<Result<_, _>>()?;

        Ok(AuthenticationMethod { algorithm, headers })
    }
}

#[derive(Debug)]
struct AuthenticationHmac(Vec<u8>);

impl FromStr for AuthenticationHmac {
    type Err = base64::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = base64::prelude::BASE64_STANDARD.decode(s)?;
        Ok(AuthenticationHmac(bytes))
    }
}

async fn get_and_update_active_session<G: CoreConfig>(
    global: &Arc<G>,
    ip_info: &IpAddressInfo,
    parts: &Parts,
    body: &[u8],
) -> Result<Option<UserSession>, StatusCode> {
    let headers = &parts.headers;

    let mut db = global.db().await.map_err(|e| {
        tracing::error!(error = %e, "failed to connect to database");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let Some(session_token_id) = get_auth_header::<UserSessionTokenId>(headers, &TOKEN_ID_HEADER)? else {
        return Ok(None);
    };
    let Some(timestamp) =
        get_auth_header::<u64>(headers, &TIMESTAMP_HEADER)?.and_then(|t| chrono::DateTime::from_timestamp_millis(t as i64))
    else {
        return Ok(None);
    };
    let Some(body_sha256) = get_auth_header::<String>(headers, &BODY_SHA256_HEADER)?
        .and_then(|h| base64::prelude::BASE64_STANDARD.decode(h).ok())
    else {
        return Ok(None);
    };

    let Some(auth_method) = get_auth_header::<AuthenticationMethod>(headers, &AUTHENTICATION_METHOD_HEADER)? else {
        return Ok(None);
    };
    let Some(auth_hmac) = get_auth_header::<AuthenticationHmac>(headers, &AUTHENTICATION_HMAC_HEADER)? else {
        return Ok(None);
    };

    if !sha2::Sha256::digest(body).as_slice().eq(&body_sha256) {
        tracing::debug!("body SHA256 mismatch");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if timestamp > chrono::Utc::now() || timestamp < chrono::Utc::now() - chrono::Duration::minutes(2) {
        tracing::debug!(timestamp = %timestamp, "invalid request timestamp");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if !auth_method.headers.contains(&TOKEN_ID_HEADER)
        || !auth_method.headers.contains(&TIMESTAMP_HEADER)
        || !auth_method.headers.contains(&NONCE_HEADER)
        || !auth_method.headers.contains(&BODY_SHA256_HEADER)
    {
        tracing::debug!("missing required headers in authentication method");
        return Err(StatusCode::BAD_REQUEST);
    }

    let Some(session) = diesel::update(user_sessions::dsl::user_sessions)
        .set((
            user_sessions::dsl::last_ip.eq(ip_info.to_network()),
            user_sessions::dsl::last_used_at.eq(chrono::Utc::now()),
        ))
        .filter(
            user_sessions::dsl::token_id
                .eq(session_token_id)
                .and(user_sessions::dsl::token.is_not_null())
                .and(user_sessions::dsl::token_expires_at.gt(chrono::Utc::now()))
                .and(user_sessions::dsl::expires_at.gt(chrono::Utc::now())),
        )
        .returning(UserSession::as_select())
        .get_results::<UserSession>(&mut db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to update user session");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_iter()
        .next()
    else {
        return Ok(None);
    };

    let token = session.token.as_ref().expect("known to be not null due to filter");

    // Verify HMAC
    match auth_method.algorithm {
        AuthenticationAlgorithm::HmacSha256 => {
            let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(token).map_err(|e| {
                tracing::error!(error = %e, "failed to create HMAC instance");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            mac.update(parts.method.as_str().as_bytes());
            mac.update(parts.uri.path().as_bytes());

            for header_name in &auth_method.headers {
                if let Some(value) = headers.get(header_name) {
                    mac.update(value.as_bytes());
                } else {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }

            mac.verify_slice(&auth_hmac.0).map_err(|e| {
                tracing::debug!(error = %e, "HMAC verification failed");
                StatusCode::UNAUTHORIZED
            })?;
        }
    }

    Ok(Some(session))
}
