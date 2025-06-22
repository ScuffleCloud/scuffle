use std::str::FromStr;
use std::sync::Arc;

use axum::extract::Request;
use axum::http::{self, HeaderMap, HeaderName, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use hmac::Mac;

use crate::CoreConfig;
use crate::models::{UserSession, UserSessionTokenId};
use crate::schema::user_sessions;

const TOKEN_ID_HEADER: HeaderName = HeaderName::from_static("scuf-token-id");
const AUTHENTICATION_METHOD_HEADER: HeaderName = HeaderName::from_static("scuf-auth-method");
const AUTHENTICATION_HMAC_HEADER: HeaderName = HeaderName::from_static("scuf-auth-hmac");

pub async fn auth<G: CoreConfig>(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let global = req.extensions().get::<Arc<G>>().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(session) = get_active_session(global, req.headers()).await? {
        // Append exntesion
        req.extensions_mut().insert(session);
    }

    Ok(next.run(req).await)
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

async fn get_active_session<G: CoreConfig>(global: &Arc<G>, headers: &HeaderMap) -> Result<Option<UserSession>, StatusCode> {
    let mut db = global.db().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(session_token_id) = get_auth_header::<UserSessionTokenId>(headers, &TOKEN_ID_HEADER)? else {
        return Ok(None);
    };
    let Some(auth_method) = get_auth_header::<AuthenticationMethod>(headers, &AUTHENTICATION_METHOD_HEADER)? else {
        return Ok(None);
    };
    let Some(auth_hmac) = get_auth_header::<AuthenticationHmac>(headers, &AUTHENTICATION_HMAC_HEADER)? else {
        return Ok(None);
    };

    if !auth_method.headers.contains(&http::header::DATE) || !auth_method.headers.contains(&TOKEN_ID_HEADER) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let Some(session) = user_sessions::dsl::user_sessions
        .filter(
            user_sessions::dsl::token_id
                .eq(session_token_id)
                .and(user_sessions::dsl::token.is_not_null())
                .and(user_sessions::dsl::token_expires_at.gt(chrono::Utc::now()))
                .and(user_sessions::dsl::expires_at.gt(chrono::Utc::now())),
        )
        .select(UserSession::as_select())
        .first::<UserSession>(&mut *db)
        .await
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(None);
    };

    let token = session.token.as_ref().expect("known to be not null due to filter");

    // Verify HMAC
    match auth_method.algorithm {
        AuthenticationAlgorithm::HmacSha256 => {
            let mut mac =
                hmac::Hmac::<sha2::Sha256>::new_from_slice(token).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            for header_name in &auth_method.headers {
                if let Some(value) = headers.get(header_name) {
                    mac.update(value.as_bytes());
                } else {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }

            mac.verify_slice(&auth_hmac.0).map_err(|_| StatusCode::UNAUTHORIZED)?;
        }
    }

    Ok(Some(session))
}
