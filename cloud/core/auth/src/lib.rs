//! The authentication service for <https://scuffle.cloud/>.
//!
//! ## Authentication
//!
//! ### SCUF-USER-V1
//!
//! The `SCUF-USER-V1` authentication method is a HMAC-SHA256 authentication method for user sessions.
//!
//! The following headers are required:
//!
//! ```
//! Authentication: SCUF-USER-V1
//! Scuf-HMAC: <hmac>
//! Scuf-Date: <date>
//! Scuf-User-ID: <user-id>
//! Scuf-Device-Fingerprint: <device-fingerprint>
//! Scuf-Request-ID: <request-id>
//! ```
//!
//! - `<hmac>` is calculated by signing the following data:
//!
//! ```
//! hmac(<token>, <request-id> + <date> + <user-id> + <device-fingerprint>)
//! ```
//!
//! - `<token>` is the session token.
//! - `<request-id>` is the unique id of the request used for idempotency.
//!
//! If the `<request-id>` has already been used the request will be rejected.
//!
//! - `<date>` is the current date and time in RFC3399 format.
//!
//! If the `<date>` is older than 30 seconds, the request will be rejected.
//!
//! - `<user-id>` is the id of the user.
//! - `<device-fingerprint>` is the fingerprint of the device.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]
