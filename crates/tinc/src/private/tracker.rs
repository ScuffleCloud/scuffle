use axum::response::IntoResponse;

use super::{
    DeserializeContent, Expected, HttpErrorResponse, HttpErrorResponseCode, HttpErrorResponseDetails,
    HttpErrorResponseRequestViolation, TrackerSharedState, ValidationError,
};

pub trait Tracker {
    type Target: Expected;

    fn allow_duplicates(&self) -> bool;
}

pub trait TrackerFor {
    type Tracker: Tracker;
}

pub trait TrackerWrapper: Tracker {
    type Tracker: Tracker;
}

pub trait TrackerDeserializer<'de>: Tracker + Sized {
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>;
}

pub trait TrackerValidation: Tracker {
    fn validate(&mut self, target: &Self::Target) -> Result<(), ValidationError>;

    #[allow(clippy::result_large_err)]
    fn validate_http(
        &mut self,
        mut state: TrackerSharedState,
        target: &Self::Target,
    ) -> Result<(), axum::response::Response> {
        state.in_scope(|| self.validate(target))?;

        if state.errors.is_empty() {
            Ok(())
        } else {
            let mut details = HttpErrorResponseDetails::default();

            for error in &state.errors {
                details.request.violations.push(HttpErrorResponseRequestViolation {
                    field: error.serde_path.as_ref(),
                    description: error.message(),
                })
            }

            Err(HttpErrorResponse {
                code: HttpErrorResponseCode::InvalidArgument.into(),
                message: "bad request",
                details,
            }
            .into_response())
        }
    }
}

impl<'de, T> TrackerDeserializer<'de> for Box<T>
where
    T: TrackerDeserializer<'de>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        self.as_mut().deserialize(value, deserializer)
    }
}

impl<T> TrackerValidation for Box<T>
where
    T: TrackerValidation,
{
    fn validate(&mut self, value: &Self::Target) -> Result<(), ValidationError> {
        self.as_mut().validate(value.as_ref())
    }
}

impl<T: Tracker> Tracker for Box<T> {
    type Target = Box<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.as_ref().allow_duplicates()
    }
}

impl<T: TrackerFor> TrackerFor for Box<T> {
    type Tracker = Box<T::Tracker>;
}

impl<T: TrackerWrapper> TrackerWrapper for Box<T> {
    type Tracker = T::Tracker;
}
