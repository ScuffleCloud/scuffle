//! Core/Authentication server for <https://scuffle.cloud/>.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`

scuffle_bootstrap::main! {
    scufflecloud_core::Global {
        scuffle_signal::SignalSvc,
        scufflecloud_core::service::CoreSvc,
    }
}
