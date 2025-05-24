use std::process::{Command, Stdio};

use anyhow::Context;

use super::utils::XTaskWorkspaceMetadata;
use crate::utils;

#[derive(Debug, Clone, clap::Parser)]
pub struct Publish {}

#[derive(serde_derive::Deserialize)]
struct PrView {
    number: u32,
    state: String,
}

impl Publish {
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = utils::metadata().context("metadata")?;

        Ok(())
    }
}
