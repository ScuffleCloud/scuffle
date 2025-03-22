#[derive(Debug, thiserror::Error)]
pub enum ChunkReadError {
    #[error("missing previous chunk header: {0}")]
    MissingPreviousChunkHeader(u32),
    #[error("too many partial chunks")]
    TooManyPartialChunks,
    #[error("too many previous chunk headers")]
    TooManyPreviousChunkHeaders,
    #[error("partial chunk too large: {0}")]
    PartialChunkTooLarge(usize),
    #[error("timestamp overflow: timestamp: {0}, delta: {1}")]
    TimestampOverflow(u32, u32),
}
