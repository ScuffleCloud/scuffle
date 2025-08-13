use std::str::FromStr;

use axum::response::IntoResponse;
use bytes::Buf;
use http_body_util::BodyExt;

use crate::__private::{
    BytesLikeTracker, HttpErrorResponse, HttpErrorResponseCode, OptionalTracker, PrimitiveTracker, Tracker,
    TrackerDeserializer, TrackerSharedState, deserialize_tracker_target,
};

pub async fn deserialize_body_json<T, B>(
    parts: &http::request::Parts,
    body: B,
    tracker: &mut T,
    target: &mut T::Target,
    state: &mut TrackerSharedState,
) -> Result<(), axum::response::Response>
where
    T: for<'de> TrackerDeserializer<'de>,
    B: http_body::Body,
    B::Error: std::fmt::Display,
{
    let Some(content_type) = parts.headers.get(http::header::CONTENT_TYPE) else {
        return Ok(());
    };

    let content_type = content_type.to_str().map_err(|_| {
        HttpErrorResponse {
            code: HttpErrorResponseCode::InvalidArgument,
            details: Default::default(),
            message: "content-type header is not valid utf-8",
        }
        .into_response()
    })?;

    let content_type = mediatype::MediaTypeBuf::from_str(content_type).map_err(|err| {
        HttpErrorResponse {
            code: HttpErrorResponseCode::InvalidArgument,
            details: Default::default(),
            message: &format!("content-type header is not valid: {err}"),
        }
        .into_response()
    })?;

    if content_type.essence() != mediatype::media_type!(APPLICATION / JSON) {
        return Err(HttpErrorResponse {
            code: HttpErrorResponseCode::InvalidArgument,
            details: Default::default(),
            message: "content-type header is not application/json",
        }
        .into_response());
    }

    let body = body
        .collect()
        .await
        .map_err(|err| {
            HttpErrorResponse {
                code: HttpErrorResponseCode::InvalidArgument,
                details: Default::default(),
                message: &format!("failed to read body: {err}"),
            }
            .into_response()
        })?
        .aggregate();

    let mut de = serde_json::Deserializer::from_reader(body.reader());

    if let Err(err) = deserialize_tracker_target(state, &mut de, tracker, target) {
        return Err(HttpErrorResponse {
            code: HttpErrorResponseCode::InvalidArgument,
            details: Default::default(),
            message: &format!("failed to deserialize body: {err}"),
        }
        .into_response());
    }

    Ok(())
}

impl<T> BytesLikeTracker for OptionalTracker<T>
where
    T: BytesLikeTracker + Default,
    T::Target: Default,
{
    fn set_target(&mut self, target: &mut Self::Target, buf: impl Buf) {
        self.0.get_or_insert_default().set_target(target.get_or_insert_default(), buf);
    }
}

pub async fn deserialize_body_bytes<T, B>(
    _: &http::request::Parts,
    body: B,
    tracker: &mut T,
    target: &mut T::Target,
    _: &mut TrackerSharedState,
) -> Result<(), axum::response::Response>
where
    T: BytesLikeTracker,
    B: http_body::Body,
    B::Error: std::fmt::Debug,
{
    let buf = body
        .collect()
        .await
        .map_err(|err| {
            HttpErrorResponse {
                code: HttpErrorResponseCode::InvalidArgument,
                details: Default::default(),
                message: &format!("failed to read body: {err:?}"),
            }
            .into_response()
        })?
        .aggregate();

    tracker.set_target(target, buf);

    Ok(())
}

pub trait TextLikeTracker: Tracker {
    fn set_target(&mut self, target: &mut Self::Target, body: String);
}

impl<T> TextLikeTracker for OptionalTracker<T>
where
    T: TextLikeTracker + Default,
    T::Target: Default,
{
    fn set_target(&mut self, target: &mut Self::Target, body: String) {
        self.0
            .get_or_insert_default()
            .set_target(target.get_or_insert_default(), body);
    }
}

impl TextLikeTracker for PrimitiveTracker<String> {
    fn set_target(&mut self, target: &mut Self::Target, body: String) {
        *target = body;
    }
}

pub async fn deserialize_body_text<T, B>(
    _: &http::request::Parts,
    body: B,
    tracker: &mut T,
    target: &mut T::Target,
    _: &mut TrackerSharedState,
) -> Result<(), axum::response::Response>
where
    T: TextLikeTracker,
    B: http_body::Body,
    B::Error: std::fmt::Debug,
{
    let mut buf = body
        .collect()
        .await
        .map_err(|err| {
            HttpErrorResponse {
                code: HttpErrorResponseCode::InvalidArgument,
                details: Default::default(),
                message: &format!("failed to read body: {err:?}"),
            }
            .into_response()
        })?
        .aggregate();

    let mut vec = vec![0; buf.remaining()];
    buf.copy_to_slice(&mut vec);

    let string = String::from_utf8(vec).map_err(|err| {
        HttpErrorResponse {
            code: HttpErrorResponseCode::InvalidArgument,
            details: Default::default(),
            message: &format!("failed to read body: {err:?}"),
        }
        .into_response()
    })?;

    tracker.set_target(target, string);

    Ok(())
}
