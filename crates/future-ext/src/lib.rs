//! A crate that extends the `Future` trait with additional methods.
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// The [`FutureExt`] trait is a trait that provides a more ergonomic way to
/// extend futures with additional functionality. Similar to the `IteratorExt`
/// trait from the `itertools` crate, but for futures.
pub trait FutureExt {
    /// Attach a timeout to the future.
    ///
    /// This is a convenience method that wraps the [`tokio::time::timeout`]
    /// function. The future will automatically cancel after the timeout has
    /// elapsed. This is equivalent to calling
    /// `with_timeout_at(tokio::time::Instant::now() + duration)`.
    fn with_timeout(self, duration: tokio::time::Duration) -> tokio::time::Timeout<Self>
    where
        Self: Sized;

    /// Attach a timeout to the future.
    ///
    /// This is a convenience method that wraps the [`tokio::time::timeout_at`]
    /// function. The future will automatically cancel after the timeout has
    /// elapsed. Unlike the `with_timeout` method, this method allows you to
    /// specify a deadline instead of a duration.
    fn with_timeout_at(self, deadline: tokio::time::Instant) -> tokio::time::Timeout<Self>
    where
        Self: Sized;
}

impl<F: std::future::Future> FutureExt for F {
    fn with_timeout(self, duration: tokio::time::Duration) -> tokio::time::Timeout<Self> {
        tokio::time::timeout(duration, self)
    }

    fn with_timeout_at(self, deadline: tokio::time::Instant) -> tokio::time::Timeout<Self> {
        tokio::time::timeout_at(deadline, self)
    }
}
