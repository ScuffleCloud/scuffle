//! Generating HLS playlists.
#![cfg_attr(feature = "docs", doc = "\n\nSee the [changelog][changelog] for a full release history.")]
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use std::io;

use crate::basic::ExtVersion;

pub mod attribute_name;
pub mod basic;
// pub mod master_playlist;
pub mod media_playlist;
pub mod media_segment;

pub use attribute_name::AttributeName;

pub trait Tag {
    const NAME: &'static str;

    fn min_version(&self) -> ExtVersion {
        ExtVersion::default()
    }

    fn write_value(&self, writer: impl io::Write) -> Result<(), io::Error> {
        let _ = writer;
        Ok(())
    }

    fn write(&self, mut writer: impl io::Write) -> Result<(), io::Error> {
        writer.write_all(b"#")?;
        writer.write_all(Self::NAME.as_bytes())?;
        self.write_value(&mut writer)?;
        writer.write_all(b"\n")?;
        Ok(())
    }
}
