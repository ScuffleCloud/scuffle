use std::str::FromStr;

use axum::response::IntoResponse;
use bytes::{Buf, Bytes};
use http_body_util::BodyExt;

use crate::__private::{
    HttpErrorResponse, HttpErrorResponseCode, OptionalTracker, PrimitiveTracker, RepeatedVecTracker, Tracker,
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

pub trait BytesLikeTracker: Tracker {
    fn set_target(&mut self, target: &mut Self::Target, buf: impl Buf);
}

impl BytesLikeTracker for PrimitiveTracker<Bytes> {
    fn set_target(&mut self, target: &mut Self::Target, mut buf: impl Buf) {
        *target = buf.copy_to_bytes(buf.remaining());
    }
}
impl BytesLikeTracker for RepeatedVecTracker<PrimitiveTracker<u8>> {
    fn set_target(&mut self, target: &mut Self::Target, mut buf: impl Buf) {
        target.clear();
        target.reserve_exact(buf.remaining());
        while buf.has_remaining() {
            let chunk = buf.chunk();
            target.extend_from_slice(chunk);
            buf.advance(chunk.len());
        }
    }
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
