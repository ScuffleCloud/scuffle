use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum PathItem {
    Field(&'static str),
    Index(usize),
    Key(MapKey),
}

pub struct PathAllowerToken {
    _marker: PhantomData<()>,
}

impl PathAllowerToken {
    pub fn push<E>(field: &'static str) -> Result<Self, E>
    where
        E: serde::de::Error,
    {
        PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(field);
        });
        Ok(Self { _marker: PhantomData })
    }
}

impl Drop for PathAllowerToken {
    fn drop(&mut self) {
        PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().pop();
        });
    }
}

pub struct PathToken<'a> {
    previous: Option<PathItem>,
    _marker: PhantomData<&'a ()>,
    _no_send: PhantomData<*const ()>,
}

fn current_path() -> String {
    ERROR_PATH_BUFFER.with(|buffer| {
        let mut path = String::new();
        for token in buffer.borrow().iter() {
            match token {
                PathItem::Field(field) => {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(field);
                }
                PathItem::Key(key) => {
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(&key.0.to_string());
                }
                PathItem::Index(index) => {
                    path.push('[');
                    path.push_str(&index.to_string());
                    path.push(']');
                }
            }
        }

        path
    })
}

pub fn report_serde_error<E>(error: E) -> Result<(), E>
where
    E: serde::de::Error,
{
    STATE.with_borrow_mut(|state| {
        if let Some(state) = state {
            if state.irrecoverable || state.unwinding {
                state.unwinding = true;
                return Err(error);
            }

            state
                .inner
                .errors
                .push(TrackedError::invalid_field(error.to_string().into_boxed_str()));

            if state.inner.fail_fast {
                state.unwinding = true;
                Err(error)
            } else {
                Ok(())
            }
        } else {
            Err(error)
        }
    })
}

pub fn report_error<E>(error: TrackedError) -> Result<(), E>
where
    E: serde::de::Error,
{
    STATE.with_borrow_mut(|state| {
        if let Some(state) = state {
            if state.irrecoverable || state.unwinding {
                state.unwinding = true;
                return Err(E::custom(&error));
            }

            let result = if state.inner.fail_fast && error.fatal {
                state.unwinding = true;
                Err(E::custom(&error))
            } else {
                Ok(())
            };

            state.inner.errors.push(error);

            result
        } else if error.fatal {
            Err(E::custom(&error))
        } else {
            Ok(())
        }
    })
}

#[inline(always)]
pub fn is_path_allowed() -> bool {
    true
}

#[track_caller]
pub fn set_irrecoverable() {
    STATE.with_borrow_mut(|state| {
        if let Some(state) = state {
            state.irrecoverable = true;
        }
    });
}

impl<'a> PathToken<'a> {
    pub fn push_field(field: &'a str) -> Self {
        ERROR_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Field(
                // SAFETY: `field` has a lifetime of `'a`, field-name hides the field so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this field after its lifetime ends.
                unsafe { std::mem::transmute::<&'a str, &'static str>(field) },
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
            previous: None,
        }
    }

    pub fn replace_field(field: &'a str) -> Self {
        let previous = ERROR_PATH_BUFFER.with(|buffer| buffer.borrow_mut().pop());
        Self {
            previous,
            ..Self::push_field(field)
        }
    }

    pub fn push_index(index: usize) -> Self {
        ERROR_PATH_BUFFER.with(|buffer| buffer.borrow_mut().push(PathItem::Index(index)));
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
            previous: None,
        }
    }

    pub fn push_key(key: &'a dyn std::fmt::Display) -> Self {
        ERROR_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Key(
                // SAFETY: `key` has a lifetime of `'a`, map-key hides the key so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this key after its lifetime ends.
                MapKey(unsafe { std::mem::transmute::<&'a dyn std::fmt::Display, &'static dyn std::fmt::Display>(key) }),
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
            previous: None,
        }
    }
}

impl Drop for PathToken<'_> {
    fn drop(&mut self) {
        ERROR_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().pop();
            if let Some(previous) = self.previous.take() {
                buffer.borrow_mut().push(previous);
            }
        });
    }
}

thread_local! {
    static ERROR_PATH_BUFFER: RefCell<Vec<PathItem>> = const { RefCell::new(Vec::new()) };
    static PATH_BUFFER: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    static STATE: RefCell<Option<InternalTrackerState>> = const { RefCell::new(None) };
}

struct InternalTrackerState {
    irrecoverable: bool,
    unwinding: bool,
    inner: TrackerSharedState,
}

struct TrackerStateGuard<'a> {
    state: &'a mut TrackerSharedState,
    _no_send: PhantomData<*const ()>,
}

impl<'a> TrackerStateGuard<'a> {
    fn new(state: &'a mut TrackerSharedState) -> Self {
        STATE.with_borrow_mut(|current| {
            if current.is_none() {
                *current = Some(InternalTrackerState {
                    irrecoverable: false,
                    unwinding: false,
                    inner: std::mem::take(state),
                });
            } else {
                panic!("TrackerStateGuard: already in use");
            }
            TrackerStateGuard {
                state,
                _no_send: PhantomData,
            }
        })
    }
}

impl Drop for TrackerStateGuard<'_> {
    fn drop(&mut self) {
        STATE.with_borrow_mut(|state| {
            if let Some(InternalTrackerState { inner, .. }) = state.take() {
                *self.state = inner;
            } else {
                panic!("TrackerStateGuard: already dropped");
            }
        });
    }
}

#[derive(Debug)]
pub enum TrackedErrorKind {
    DuplicateField,
    UnknownField,
    MissingField,
    InvalidField { message: Box<str> },
}

#[derive(Debug)]
pub struct TrackedError {
    pub kind: TrackedErrorKind,
    pub fatal: bool,
    pub path: Box<str>,
}

impl std::fmt::Display for TrackedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TrackedErrorKind::DuplicateField => write!(f, "`{}` was already provided", self.path),
            TrackedErrorKind::UnknownField => write!(f, "unknown field `{}`", self.path),
            TrackedErrorKind::MissingField => write!(f, "missing field `{}`", self.path),
            TrackedErrorKind::InvalidField { message } => write!(f, "`{}`: {}", self.path, message),
        }
    }
}

impl TrackedError {
    fn new(kind: TrackedErrorKind, fatal: bool) -> Self {
        Self {
            kind,
            fatal,
            path: current_path().into_boxed_str(),
        }
    }

    pub fn unknown_field(fatal: bool) -> Self {
        Self::new(TrackedErrorKind::UnknownField, fatal)
    }

    pub fn invalid_field(message: impl Into<Box<str>>) -> Self {
        Self::new(TrackedErrorKind::InvalidField { message: message.into() }, true)
    }

    pub fn duplicate_field() -> Self {
        Self::new(TrackedErrorKind::DuplicateField, true)
    }

    pub fn missing_field() -> Self {
        Self::new(TrackedErrorKind::MissingField, true)
    }
}

#[derive(Default, Debug)]
pub struct TrackerSharedState {
    pub fail_fast: bool,
    pub errors: Vec<TrackedError>,
}

impl TrackerSharedState {
    pub fn in_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = TrackerStateGuard::new(self);
        f()
    }
}

pub struct MapKey(&'static dyn std::fmt::Display);

impl std::fmt::Debug for MapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapKey({})", self.0)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct HttpErrorResponse<'a> {
    pub message: &'a str,
    pub code: HttpErrorResponseCode,
    #[serde(skip_serializing_if = "is_default")]
    pub details: HttpErrorResponseDetails<'a>,
}

impl axum::response::IntoResponse for HttpErrorResponse<'_> {
    fn into_response(self) -> axum::response::Response {
        let status = self.code.to_http_status();
        (status, axum::Json(self)).into_response()
    }
}

#[derive(Debug)]
pub struct HttpErrorResponseCode(pub tonic::Code);

impl serde::Serialize for HttpErrorResponseCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        i32::from(self.0).serialize(serializer).map_err(serde::ser::Error::custom)
    }
}

impl HttpErrorResponseCode {
    pub fn to_http_status(&self) -> http::StatusCode {
        match self.0 {
            tonic::Code::Aborted => http::StatusCode::from_u16(499).unwrap_or(http::StatusCode::BAD_REQUEST),
            tonic::Code::Cancelled => http::StatusCode::from_u16(499).unwrap_or(http::StatusCode::BAD_REQUEST),
            tonic::Code::AlreadyExists => http::StatusCode::ALREADY_REPORTED,
            tonic::Code::DataLoss => http::StatusCode::INTERNAL_SERVER_ERROR,
            tonic::Code::DeadlineExceeded => http::StatusCode::GATEWAY_TIMEOUT,
            tonic::Code::FailedPrecondition => http::StatusCode::PRECONDITION_FAILED,
            tonic::Code::Internal => http::StatusCode::INTERNAL_SERVER_ERROR,
            tonic::Code::InvalidArgument => http::StatusCode::BAD_REQUEST,
            tonic::Code::NotFound => http::StatusCode::NOT_FOUND,
            tonic::Code::OutOfRange => http::StatusCode::BAD_REQUEST,
            tonic::Code::PermissionDenied => http::StatusCode::FORBIDDEN,
            tonic::Code::ResourceExhausted => http::StatusCode::TOO_MANY_REQUESTS,
            tonic::Code::Unauthenticated => http::StatusCode::UNAUTHORIZED,
            tonic::Code::Unavailable => http::StatusCode::SERVICE_UNAVAILABLE,
            tonic::Code::Unimplemented => http::StatusCode::NOT_IMPLEMENTED,
            tonic::Code::Unknown => http::StatusCode::INTERNAL_SERVER_ERROR,
            tonic::Code::Ok => http::StatusCode::OK,
        }
    }
}

impl From<tonic::Code> for HttpErrorResponseCode {
    fn from(code: tonic::Code) -> Self {
        Self(code)
    }
}

fn is_default<T>(t: &T) -> bool
where
    T: Default + PartialEq,
{
    t == &T::default()
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseDetails<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub retry: HttpErrorResponseRetry,
    #[serde(skip_serializing_if = "is_default")]
    pub debug: HttpErrorResponseDebug<'a>,
    #[serde(skip_serializing_if = "is_default")]
    pub quota: Vec<HttpErrorResponseQuotaViolation<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    pub error: HttpErrorResponseError<'a>,
    #[serde(skip_serializing_if = "is_default")]
    pub precondition: Vec<HttpErrorResponsePreconditionViolation<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    pub request: HttpErrorResponseRequest<'a>,
    #[serde(skip_serializing_if = "is_default")]
    pub resource: HttpErrorResponseResource<'a>,
    #[serde(skip_serializing_if = "is_default")]
    pub help: Vec<HttpErrorResponseHelpLink<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    pub localized: HttpErrorResponseLocalized<'a>,
}

impl<'a> From<&'a tonic_types::ErrorDetails> for HttpErrorResponseDetails<'a> {
    fn from(value: &'a tonic_types::ErrorDetails) -> Self {
        Self {
            retry: HttpErrorResponseRetry::from(value.retry_info()),
            debug: HttpErrorResponseDebug::from(value.debug_info()),
            quota: HttpErrorResponseQuota::from(value.quota_failure()).violations,
            error: HttpErrorResponseError::from(value.error_info()),
            precondition: HttpErrorResponsePrecondition::from(value.precondition_failure()).violations,
            request: HttpErrorResponseRequest::from((value.bad_request(), value.request_info())),
            resource: HttpErrorResponseResource::from(value.resource_info()),
            help: HttpErrorResponseHelp::from(value.help()).links,
            localized: HttpErrorResponseLocalized::from(value.localized_message()),
        }
    }
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseRetry {
    #[serde(skip_serializing_if = "is_default")]
    pub after: Option<std::time::Duration>,
    #[serde(skip_serializing_if = "is_default")]
    pub at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Option<&tonic_types::RetryInfo>> for HttpErrorResponseRetry {
    fn from(retry_info: Option<&tonic_types::RetryInfo>) -> Self {
        Self {
            after: retry_info.and_then(|ri| ri.retry_delay),
            at: retry_info.and_then(|ri| ri.retry_delay).map(|d| {
                let now = chrono::Utc::now();
                now + d
            }),
        }
    }
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseDebug<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub stack: &'a [String],
    #[serde(skip_serializing_if = "is_default")]
    pub details: &'a str,
}

impl<'a> From<Option<&'a tonic_types::DebugInfo>> for HttpErrorResponseDebug<'a> {
    fn from(debug_info: Option<&'a tonic_types::DebugInfo>) -> Self {
        Self {
            stack: debug_info.as_ref().map_or(&[], |d| &d.stack_entries),
            details: debug_info.as_ref().map_or("", |d| &d.detail),
        }
    }
}

#[derive(Default)]
pub struct HttpErrorResponseQuota<'a> {
    pub violations: Vec<HttpErrorResponseQuotaViolation<'a>>,
}

impl<'a> From<Option<&'a tonic_types::QuotaFailure>> for HttpErrorResponseQuota<'a> {
    fn from(quota_failure: Option<&'a tonic_types::QuotaFailure>) -> Self {
        Self {
            violations: quota_failure.as_ref().map_or_else(Vec::new, |qf| {
                qf.violations
                    .iter()
                    .map(|violation| HttpErrorResponseQuotaViolation {
                        subject: &violation.subject,
                        description: &violation.description,
                    })
                    .filter(|violation| !is_default(violation))
                    .collect()
            }),
        }
    }
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseQuotaViolation<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub subject: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseError<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub reason: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub domain: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub metadata: HashMap<&'a str, &'a str>,
}

impl<'a> From<Option<&'a tonic_types::ErrorInfo>> for HttpErrorResponseError<'a> {
    fn from(error_info: Option<&'a tonic_types::ErrorInfo>) -> Self {
        Self {
            reason: error_info.map_or("", |ei| ei.reason.as_str()),
            domain: error_info.map_or("", |ei| ei.domain.as_str()),
            metadata: error_info
                .map(|ei| {
                    ei.metadata
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .filter(|kv| !is_default(kv))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

pub struct HttpErrorResponsePrecondition<'a> {
    pub violations: Vec<HttpErrorResponsePreconditionViolation<'a>>,
}

impl<'a> From<Option<&'a tonic_types::PreconditionFailure>> for HttpErrorResponsePrecondition<'a> {
    fn from(precondition_failure: Option<&'a tonic_types::PreconditionFailure>) -> Self {
        Self {
            violations: precondition_failure.as_ref().map_or_else(Vec::new, |pf| {
                pf.violations
                    .iter()
                    .map(|violation| HttpErrorResponsePreconditionViolation {
                        type_: &violation.r#type,
                        subject: &violation.subject,
                        description: &violation.description,
                    })
                    .filter(|violation| !is_default(violation))
                    .collect()
            }),
        }
    }
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponsePreconditionViolation<'a> {
    #[serde(skip_serializing_if = "is_default", rename = "type")]
    pub type_: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub subject: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseRequest<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub violations: Vec<HttpErrorResponseRequestViolation<'a>>,
    #[serde(skip_serializing_if = "is_default")]
    pub id: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub serving_data: &'a str,
}

impl<'a> From<(Option<&'a tonic_types::BadRequest>, Option<&'a tonic_types::RequestInfo>)> for HttpErrorResponseRequest<'a> {
    fn from(
        (bad_request, request_info): (Option<&'a tonic_types::BadRequest>, Option<&'a tonic_types::RequestInfo>),
    ) -> Self {
        Self {
            violations: bad_request
                .as_ref()
                .map(|br| {
                    br.field_violations
                        .iter()
                        .map(|violation| HttpErrorResponseRequestViolation {
                            field: &violation.field,
                            description: &violation.description,
                        })
                        .filter(|violation| !violation.field.is_empty() && !violation.description.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            id: request_info.map_or("", |ri| ri.request_id.as_str()),
            serving_data: request_info.map_or("", |ri| ri.serving_data.as_str()),
        }
    }
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseRequestViolation<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub field: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseResource<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub name: &'a str,
    #[serde(skip_serializing_if = "is_default", rename = "type")]
    pub type_: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub owner: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

impl<'a> From<Option<&'a tonic_types::ResourceInfo>> for HttpErrorResponseResource<'a> {
    fn from(resource_info: Option<&'a tonic_types::ResourceInfo>) -> Self {
        Self {
            name: resource_info.map_or("", |ri| ri.resource_name.as_str()),
            type_: resource_info.map_or("", |ri| ri.resource_type.as_str()),
            owner: resource_info.map_or("", |ri| ri.owner.as_str()),
            description: resource_info.map_or("", |ri| ri.description.as_str()),
        }
    }
}

pub struct HttpErrorResponseHelp<'a> {
    pub links: Vec<HttpErrorResponseHelpLink<'a>>,
}

impl<'a> From<Option<&'a tonic_types::Help>> for HttpErrorResponseHelp<'a> {
    fn from(help: Option<&'a tonic_types::Help>) -> Self {
        Self {
            links: help.as_ref().map_or_else(Vec::new, |h| {
                h.links
                    .iter()
                    .map(|link| HttpErrorResponseHelpLink {
                        description: &link.description,
                        url: &link.url,
                    })
                    .filter(|link| !is_default(link))
                    .collect()
            }),
        }
    }
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseHelpLink<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub url: &'a str,
}

#[derive(Debug, serde::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseLocalized<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub locale: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub message: &'a str,
}

impl<'a> From<Option<&'a tonic_types::LocalizedMessage>> for HttpErrorResponseLocalized<'a> {
    fn from(localized_message: Option<&'a tonic_types::LocalizedMessage>) -> Self {
        Self {
            locale: localized_message.map_or("", |lm| lm.locale.as_str()),
            message: localized_message.map_or("", |lm| lm.message.as_str()),
        }
    }
}
