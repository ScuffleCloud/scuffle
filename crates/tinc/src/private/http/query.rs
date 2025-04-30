use axum::response::IntoResponse;

use crate::__private::error::HttpErrorResponse;
use crate::__private::{TrackerDeserializer, TrackerSharedState, deserialize_tracker_target};

#[allow(clippy::result_large_err)]
pub fn deserialize_query_string<'de, T>(
    parts: &'de http::request::Parts,
    tracker: &mut T,
    target: &mut T::Target,
    state: &mut TrackerSharedState,
) -> Result<(), axum::response::Response>
where
    T: TrackerDeserializer<'de>,
{
    let Some(query_string) = parts.uri.query() else {
        return Ok(());
    };

    match serde_qs::Deserializer::new(query_string.as_bytes())
        .map(|de| deserialize_tracker_target(state, de, tracker, target))
    {
        Err(err) | Ok(Err(err)) => Err(HttpErrorResponse {
            code: tonic::Code::InvalidArgument.into(),
            details: Default::default(),
            message: &format!("invalid query string: {err}"),
        }
        .into_response()),
        Ok(Ok(())) => Ok(()),
    }
}
