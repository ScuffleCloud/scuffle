use std::io;

/// Adds helfpful utilities for working with [`io::Result`].
pub trait IoResultExt<T> {
    /// Converts an [`io::ErrorKind::UnexpectedEof`] error into a [`None`] value.
    fn eof_to_none(self) -> Result<Option<T>, io::Error>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn eof_to_none(self) -> Result<Option<T>, io::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }
}
