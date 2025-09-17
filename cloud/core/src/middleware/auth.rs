use std::str::FromStr;
use std::sync::Arc;

use axum::extract::Request;
use axum::http::{HeaderMap, HeaderName, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use diesel::{BoolExpressionMethods, ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use fred::prelude::KeysInterface;
use hmac::Mac;

use crate::CoreConfig;
use crate::http_ext::RequestExt;
use crate::middleware::IpAddressInfo;
use crate::models::{UserSession, UserSessionTokenId};
use crate::schema::user_sessions;

const TOKEN_ID_HEADER: HeaderName = HeaderName::from_static("scuf-token-id");
const TIMESTAMP_HEADER: HeaderName = HeaderName::from_static("scuf-timestamp");
const NONCE_HEADER: HeaderName = HeaderName::from_static("scuf-nonce");

const AUTHENTICATION_METHOD_HEADER: HeaderName = HeaderName::from_static("scuf-auth-method");
const AUTHENTICATION_HMAC_HEADER: HeaderName = HeaderName::from_static("scuf-auth-hmac");

pub(crate) const fn auth_headers() -> [HeaderName; 5] {
    [
        TOKEN_ID_HEADER,
        TIMESTAMP_HEADER,
        NONCE_HEADER,
        AUTHENTICATION_METHOD_HEADER,
        AUTHENTICATION_HMAC_HEADER,
    ]
}

#[derive(Clone, Debug)]
pub(crate) struct ExpiredSession(pub UserSession);

pub(crate) async fn auth<G: CoreConfig>(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let global = req
        .extensions()
        .global::<G>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ip_info = req
        .extensions()
        .ip_address_info()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (session, expired_session) = get_and_update_active_session(&global, &ip_info, req.headers()).await?;
    if let Some(session) = session {
        req.extensions_mut().insert(session);
    }
    if let Some(expired_session) = expired_session {
        req.extensions_mut().insert(expired_session);
    }

    Ok(next.run(req).await)
}

fn get_auth_header<'a, T>(headers: &'a HeaderMap, header_name: &HeaderName) -> Result<Option<T>, StatusCode>
where
    T: FromStr + 'a,
    T::Err: std::fmt::Display,
{
    match headers.get(header_name) {
        Some(h) => {
            let s = h.to_str().map_err(|e| {
                tracing::debug!(header = %header_name, error = %e, "invalid header value");
                StatusCode::BAD_REQUEST
            })?;
            Ok(Some(s.parse().map_err(|e| {
                tracing::debug!(header = %header_name, error = %e, "failed to parse header value");
                StatusCode::BAD_REQUEST
            })?))
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

#[derive(thiserror::Error, Debug)]
enum NonceParseError {
    #[error("failed to decode: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("invalid nonce length {0}, must be 32 bytes")]
    InvalidLength(usize),
}

#[derive(Debug)]
struct Nonce(Vec<u8>);

impl FromStr for Nonce {
    type Err = NonceParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = base64::prelude::BASE64_STANDARD.decode(s)?;
        if bytes.len() != 32 {
            return Err(NonceParseError::InvalidLength(bytes.len()));
        }
        Ok(Nonce(bytes))
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
    headers: &HeaderMap,
) -> Result<(Option<UserSession>, Option<ExpiredSession>), StatusCode> {
    let Some(session_token_id) = get_auth_header::<UserSessionTokenId>(headers, &TOKEN_ID_HEADER)? else {
        return Ok((None, None));
    };
    let Some(timestamp) =
        get_auth_header::<u64>(headers, &TIMESTAMP_HEADER)?.and_then(|t| chrono::DateTime::from_timestamp_millis(t as i64))
    else {
        return Ok((None, None));
    };
    let Some(nonce) = get_auth_header::<Nonce>(headers, &NONCE_HEADER)? else {
        return Ok((None, None));
    };

    let Some(auth_method) = get_auth_header::<AuthenticationMethod>(headers, &AUTHENTICATION_METHOD_HEADER)? else {
        return Ok((None, None));
    };
    let Some(auth_hmac) = get_auth_header::<AuthenticationHmac>(headers, &AUTHENTICATION_HMAC_HEADER)? else {
        return Ok((None, None));
    };

    if timestamp > chrono::Utc::now() || timestamp < chrono::Utc::now() - global.timeout_config().max_request_lifetime {
        tracing::debug!(timestamp = %timestamp, "invalid request timestamp");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if !auth_method.headers.contains(&TOKEN_ID_HEADER)
        || !auth_method.headers.contains(&TIMESTAMP_HEADER)
        || !auth_method.headers.contains(&NONCE_HEADER)
    {
        tracing::debug!("missing required headers in authentication method");
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut db = global.db().await.map_err(|e| {
        tracing::error!(error = %e, "failed to connect to database");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let Some(session) = diesel::update(user_sessions::dsl::user_sessions)
        .set((
            user_sessions::dsl::last_ip.eq(ip_info.to_network()),
            user_sessions::dsl::last_used_at.eq(chrono::Utc::now()),
        ))
        .filter(
            user_sessions::dsl::token_id
                .eq(session_token_id)
                .and(user_sessions::dsl::token.is_not_null())
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
        tracing::debug!(token_id = %session_token_id, "no active session found");
        return Err(StatusCode::UNAUTHORIZED);
    };

    let token = session.token.as_ref().expect("known to be not null due to filter");

    // Verify HMAC
    match auth_method.algorithm {
        AuthenticationAlgorithm::HmacSha256 => {
            let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(token).map_err(|e| {
                tracing::error!(error = %e, "failed to create HMAC instance");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            for header_name in &auth_method.headers {
                if let Some(value) = headers.get(header_name) {
                    mac.update(value.as_bytes());
                } else {
                    tracing::debug!(header = %header_name, "missing header");
                    return Err(StatusCode::BAD_REQUEST);
                }
            }

            mac.verify_slice(&auth_hmac.0).map_err(|e| {
                tracing::debug!(error = %e, "HMAC verification failed");
                StatusCode::UNAUTHORIZED
            })?;
        }
    }

    let mut key = "nonces:".as_bytes().to_vec();
    key.extend_from_slice(&nonce.0);
    let value: Option<bool> = global
        .redis()
        .set(
            key.as_slice(),
            true,
            Some(fred::types::Expiration::PX(
                global.timeout_config().max_request_lifetime.num_milliseconds(),
            )),
            Some(fred::types::SetOptions::NX),
            true,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to set nonce in redis");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if value.is_some() {
        tracing::debug!("replayed nonce detected");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if session.token_expires_at.is_some_and(|t| t <= chrono::Utc::now()) {
        return Ok((None, Some(ExpiredSession(session))));
    }

    Ok((Some(session), None))
}
