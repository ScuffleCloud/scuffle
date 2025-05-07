use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::marker::PhantomData;

use axum::response::IntoResponse;

use super::FuncFmt;

#[derive(Debug)]
pub enum PathItem {
    Field(&'static str),
    Index(usize),
    Key(MapKey),
}

pub struct ProtoPathToken<'a> {
    _no_send: PhantomData<*const ()>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ProtoPathToken<'a> {
    pub fn push_field(field: &'a str) -> Self {
        PROTO_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Field(
                // SAFETY: `field` has a lifetime of `'a`, field-name hides the field so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this field after its lifetime ends.
                unsafe { std::mem::transmute::<&'a str, &'static str>(field) },
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
        }
    }

    pub fn push_index(index: usize) -> Self {
        PROTO_PATH_BUFFER.with(|buffer| buffer.borrow_mut().push(PathItem::Index(index)));
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
        }
    }

    pub fn push_key(key: &'a dyn std::fmt::Debug) -> Self {
        PROTO_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Key(
                // SAFETY: `key` has a lifetime of `'a`, map-key hides the key so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this key after its lifetime ends.
                MapKey(unsafe { std::mem::transmute::<&'a dyn std::fmt::Debug, &'static dyn std::fmt::Debug>(key) }),
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
        }
    }

    pub fn current_path() -> String {
        PROTO_PATH_BUFFER.with(|buffer| format_path_items(buffer.borrow().as_slice()))
    }
}

impl Drop for ProtoPathToken<'_> {
    fn drop(&mut self) {
        PROTO_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().pop();
        });
    }
}

pub struct SerdePathToken<'a> {
    previous: Option<PathItem>,
    _marker: PhantomData<&'a ()>,
    _no_send: PhantomData<*const ()>,
}

pub fn report_de_error<E>(error: E) -> Result<(), E>
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

pub fn report_tracked_error<E>(error: TrackedError) -> Result<(), E>
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

impl<'a> SerdePathToken<'a> {
    pub fn push_field(field: &'a str) -> Self {
        SERDE_PATH_BUFFER.with(|buffer| {
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
        let previous = SERDE_PATH_BUFFER.with(|buffer| buffer.borrow_mut().pop());
        Self {
            previous,
            ..Self::push_field(field)
        }
    }

    pub fn push_index(index: usize) -> Self {
        SERDE_PATH_BUFFER.with(|buffer| buffer.borrow_mut().push(PathItem::Index(index)));
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
            previous: None,
        }
    }

    pub fn push_key(key: &'a dyn std::fmt::Debug) -> Self {
        SERDE_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().push(PathItem::Key(
                // SAFETY: `key` has a lifetime of `'a`, map-key hides the key so it cannot be accessed outside of this module.
                // We return a `PathToken` that has a lifetime of `'a` which makes it impossible to access this key after its lifetime ends.
                MapKey(unsafe { std::mem::transmute::<&'a dyn std::fmt::Debug, &'static dyn std::fmt::Debug>(key) }),
            ))
        });
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
            previous: None,
        }
    }

    pub fn current_path() -> String {
        SERDE_PATH_BUFFER.with(|buffer| format_path_items(buffer.borrow().as_slice()))
    }
}

fn format_path_items(items: &[PathItem]) -> String {
    FuncFmt(|fmt| {
        let mut first = true;
        for token in items {
            match token {
                PathItem::Field(field) => {
                    if !first {
                        fmt.write_char('.')?;
                    }
                    first = false;
                    fmt.write_str(field)?;
                }
                PathItem::Index(index) => {
                    fmt.write_char('[')?;
                    std::fmt::Display::fmt(index, fmt)?;
                    fmt.write_char(']')?;
                }
                PathItem::Key(key) => {
                    fmt.write_char('[')?;
                    key.0.fmt(fmt)?;
                    fmt.write_char(']')?;
                }
            }
        }

        Ok(())
    })
    .to_string()
}

impl Drop for SerdePathToken<'_> {
    fn drop(&mut self) {
        SERDE_PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().pop();
            if let Some(previous) = self.previous.take() {
                buffer.borrow_mut().push(previous);
            }
        });
    }
}

thread_local! {
    static SERDE_PATH_BUFFER: RefCell<Vec<PathItem>> = const { RefCell::new(Vec::new()) };
    static PROTO_PATH_BUFFER: RefCell<Vec<PathItem>> = const { RefCell::new(Vec::new()) };
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

impl TrackedError {
    pub fn message(&self) -> &str {
        match &self.kind {
            TrackedErrorKind::DuplicateField => "duplicate field",
            TrackedErrorKind::UnknownField => "unknown field",
            TrackedErrorKind::MissingField => "missing field",
            TrackedErrorKind::InvalidField { message } => message.as_ref(),
        }
    }
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
            path: match tinc_cel::CelMode::current() {
                tinc_cel::CelMode::Json => SerdePathToken::current_path().into_boxed_str(),
                tinc_cel::CelMode::Proto => ProtoPathToken::current_path().into_boxed_str(),
            },
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

pub struct MapKey(&'static dyn std::fmt::Debug);

impl std::fmt::Debug for MapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapKey({:?})", self.0)
    }
}

#[cfg(feature = "tonic")]
pub fn handle_tonic_status(status: &tonic::Status) -> axum::response::Response {
    use tonic_types::StatusExt;

    let code = HttpErrorResponseCode::from(status.code());
    let details = status.get_error_details();
    let details = HttpErrorResponseDetails::from(&details);
    HttpErrorResponse {
        message: status.message(),
        code,
        details,
    }
    .into_response()
}

pub fn handle_response_build_error(err: impl std::error::Error) -> axum::response::Response {
    HttpErrorResponse {
        message: &err.to_string(),
        code: HttpErrorResponseCode::Internal,
        details: Default::default(),
    }
    .into_response()
}

#[derive(Debug, serde_derive::Serialize)]
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
pub enum HttpErrorResponseCode {
    Aborted,
    Cancelled,
    AlreadyExists,
    DataLoss,
    DeadlineExceeded,
    FailedPrecondition,
    Internal,
    InvalidArgument,
    NotFound,
    OutOfRange,
    PermissionDenied,
    ResourceExhausted,
    Unauthenticated,
    Unavailable,
    Unimplemented,
    Unknown,
    Ok,
}

impl serde::Serialize for HttpErrorResponseCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_http_status()
            .as_str()
            .serialize(serializer)
            .map_err(serde::ser::Error::custom)
    }
}

impl HttpErrorResponseCode {
    pub fn to_http_status(&self) -> http::StatusCode {
        match self {
            Self::Aborted => http::StatusCode::from_u16(499).unwrap_or(http::StatusCode::BAD_REQUEST),
            Self::Cancelled => http::StatusCode::from_u16(499).unwrap_or(http::StatusCode::BAD_REQUEST),
            Self::AlreadyExists => http::StatusCode::ALREADY_REPORTED,
            Self::DataLoss => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::DeadlineExceeded => http::StatusCode::GATEWAY_TIMEOUT,
            Self::FailedPrecondition => http::StatusCode::PRECONDITION_FAILED,
            Self::Internal => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidArgument => http::StatusCode::BAD_REQUEST,
            Self::NotFound => http::StatusCode::NOT_FOUND,
            Self::OutOfRange => http::StatusCode::BAD_REQUEST,
            Self::PermissionDenied => http::StatusCode::FORBIDDEN,
            Self::ResourceExhausted => http::StatusCode::TOO_MANY_REQUESTS,
            Self::Unauthenticated => http::StatusCode::UNAUTHORIZED,
            Self::Unavailable => http::StatusCode::SERVICE_UNAVAILABLE,
            Self::Unimplemented => http::StatusCode::NOT_IMPLEMENTED,
            Self::Unknown => http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Ok => http::StatusCode::OK,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<tonic::Code> for HttpErrorResponseCode {
    fn from(code: tonic::Code) -> Self {
        match code {
            tonic::Code::Aborted => Self::Aborted,
            tonic::Code::Cancelled => Self::Cancelled,
            tonic::Code::AlreadyExists => Self::AlreadyExists,
            tonic::Code::DataLoss => Self::DataLoss,
            tonic::Code::DeadlineExceeded => Self::DeadlineExceeded,
            tonic::Code::FailedPrecondition => Self::FailedPrecondition,
            tonic::Code::Internal => Self::Internal,
            tonic::Code::InvalidArgument => Self::InvalidArgument,
            tonic::Code::NotFound => Self::NotFound,
            tonic::Code::OutOfRange => Self::OutOfRange,
            tonic::Code::PermissionDenied => Self::PermissionDenied,
            tonic::Code::ResourceExhausted => Self::ResourceExhausted,
            tonic::Code::Unauthenticated => Self::Unauthenticated,
            tonic::Code::Unavailable => Self::Unavailable,
            tonic::Code::Unimplemented => Self::Unimplemented,
            tonic::Code::Unknown => Self::Unknown,
            tonic::Code::Ok => Self::Ok,
        }
    }
}

fn is_default<T>(t: &T) -> bool
where
    T: Default + PartialEq,
{
    t == &T::default()
}

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseQuotaViolation<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub subject: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
pub struct HttpErrorResponsePreconditionViolation<'a> {
    #[serde(skip_serializing_if = "is_default", rename = "type")]
    pub type_: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub subject: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseRequestViolation<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub field: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
}

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
pub struct HttpErrorResponseHelpLink<'a> {
    #[serde(skip_serializing_if = "is_default")]
    pub description: &'a str,
    #[serde(skip_serializing_if = "is_default")]
    pub url: &'a str,
}

#[derive(Debug, serde_derive::Serialize, Default, PartialEq)]
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
