//! Config parsing.

/// This trait is used to parse a configuration for the application.
///
/// The avoid having to manually implement this trait, the `bootstrap!` macro in
/// the [`scuffle-settings`] crate can be used to
/// generate an implementation.
///
/// # See Also
///
/// - [`Global`](crate::Global)
/// - [`scuffle-settings`]
///
/// [scuffle-settings]: ../scuffle-settings
pub trait ConfigParser: Sized {
    /// Parse the configuration for the application.
    fn parse() -> impl std::future::Future<Output = anyhow::Result<Self>>;
}

impl ConfigParser for () {
    #[inline(always)]
    fn parse() -> impl std::future::Future<Output = anyhow::Result<Self>> {
        std::future::ready(Ok(()))
    }
}

/// An empty configuration that can be used when no configuration is needed.
pub struct EmptyConfig;

impl ConfigParser for EmptyConfig {
    #[inline(always)]
    fn parse() -> impl std::future::Future<Output = anyhow::Result<Self>> {
        std::future::ready(Ok(EmptyConfig))
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::{ConfigParser, EmptyConfig};

    #[tokio::test]
    async fn unit_config() {
        assert!(matches!(<()>::parse().await, Ok(())));
    }

    #[tokio::test]
    async fn empty_config() {
        assert!(matches!(EmptyConfig::parse().await, Ok(EmptyConfig)));
    }
}
