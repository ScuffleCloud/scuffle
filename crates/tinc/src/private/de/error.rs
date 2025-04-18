use std::cell::RefCell;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum PathItem {
    Field(&'static str),
    Index(usize),
    Key(MapKey),
}

pub struct PathToken<'a> {
    previous: Option<PathItem>,
    _marker: PhantomData<&'a ()>,
    _no_send: PhantomData<*const ()>,
}

fn current_path() -> String {
    PATH_BUFFER.with(|buffer| {
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

pub fn report_error<E>(error: TrackedError) -> Result<(), E>
where
    E: serde::de::Error,
{
    STATE.with_borrow_mut(|state| {
        if let Some(state) = state {
            if state.irrecoverable {
                return Err(E::custom(&error));
            }

            let result = if state.fail_fast && error.fatal {
                Err(E::custom(&error))
            } else {
                Ok(())
            };

            state.errors.push(error);
            result
        } else {
            Err(E::custom(&error))
        }
    })
}

pub fn is_path_allowed() -> bool {
    PATH_BUFFER
        .with(|buffer| STATE.with_borrow(|settings| settings.as_ref().is_none_or(|s| (s.path_allowed)(&buffer.borrow()))))
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
        PATH_BUFFER.with(|buffer| {
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
        let previous = PATH_BUFFER.with(|buffer| buffer.borrow_mut().pop());
        Self {
            previous,
            ..Self::push_field(field)
        }
    }

    pub fn push_index(index: usize) -> Self {
        PATH_BUFFER.with(|buffer| buffer.borrow_mut().push(PathItem::Index(index)));
        Self {
            _marker: PhantomData,
            _no_send: PhantomData,
            previous: None,
        }
    }

    pub fn push_key(key: &'a dyn std::fmt::Display) -> Self {
        PATH_BUFFER.with(|buffer| {
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
        PATH_BUFFER.with(|buffer| {
            buffer.borrow_mut().pop();
            if let Some(previous) = self.previous.take() {
                buffer.borrow_mut().push(previous);
            }
        });
    }
}

thread_local! {
    static PATH_BUFFER: RefCell<Vec<PathItem>> = const { RefCell::new(Vec::new()) };
    static STATE: RefCell<Option<TrackerSharedState>> = const {
        RefCell::new(None)
    };
}

struct TrackerStateGuard<'a> {
    state: &'a mut TrackerSharedState,
    _no_send: PhantomData<*const ()>,
}

impl<'a> TrackerStateGuard<'a> {
    fn new(state: &'a mut TrackerSharedState) -> Self {
        STATE.with_borrow_mut(|current| {
            if current.is_none() {
                *current = Some(std::mem::take(state));
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
            if let Some(current) = state.take() {
                *self.state = current;
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

pub struct TrackerSharedState {
    pub fail_fast: bool,
    pub irrecoverable: bool,
    pub errors: Vec<TrackedError>,
    pub path_allowed: fn(&[PathItem]) -> bool,
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

impl std::fmt::Debug for TrackerSharedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("TrackerSharedState");
        s.field("fail_fast", &self.fail_fast);
        s.field("irrecoverable", &self.irrecoverable);
        s.field("errors", &self.errors);
        s.finish()
    }
}

impl Default for TrackerSharedState {
    fn default() -> Self {
        Self {
            fail_fast: true,
            irrecoverable: false,
            errors: Vec::new(),
            path_allowed: |_| true,
        }
    }
}

pub struct MapKey(&'static dyn std::fmt::Display);

impl std::fmt::Debug for MapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapKey({})", self.0)
    }
}
