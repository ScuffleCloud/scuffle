#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("error validating field `{field}`: {error}")]
    Expression {
        field: String,
        error: String,
        expression: &'static str,
        this: Option<&'static str>,
        input: &'static str,
    },
}

pub trait ValidateMessage {
    fn validate(&self) -> Result<(), ValidationError>;
}
