#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("error evaluating expression `{expression}` on field `{field}`: {error}")]
    Expression {
        field: Box<str>,
        error: Box<str>,
        expression: &'static str,
    },
    #[error("{0}")]
    FailFast(Box<str>),
}

impl serde::de::Error for ValidationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::FailFast(msg.to_string().into_boxed_str())
    }
}

pub trait ValidateMessage {
    fn validate(&self) -> Result<(), ValidationError>;
}
