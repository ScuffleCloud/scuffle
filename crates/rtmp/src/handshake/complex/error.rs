#[derive(Debug, thiserror::Error)]
pub enum ComplexHandshakeError {
    #[error("not enough data")]
    NotEnoughData,
    #[error("digest length not correct")]
    DigestLengthNotCorrect,
    #[error("cannot generate digest")]
    CannotGenerate,
    #[error("unknown schema")]
    UnknownSchema,
}
