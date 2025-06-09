//! Core/Authentication server for <https://scuffle.cloud/>.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`

use std::net::SocketAddr;

pub mod schema;
pub mod service;

pub trait CoreGlobal: scuffle_bootstrap::Global + scuffle_signal::SignalConfig {
    fn bind(&self) -> SocketAddr;
}
