use crate::chunk::ChunkWriteError;

#[derive(Debug, thiserror::Error)]
pub enum EventMessagesError {
    #[error("chunk write error: {0}")]
    ChunkWrite(#[from] ChunkWriteError),
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = EventMessagesError::ChunkWrite(ChunkWriteError::UnknownReadState);
        assert_eq!(format!("{}", error), "chunk write error: unknown read state");
    }
}
