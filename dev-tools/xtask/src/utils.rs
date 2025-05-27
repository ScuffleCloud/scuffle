use std::ffi::OsStr;
use std::fmt::{Display, Write};

use anyhow::Context;

pub fn metadata() -> anyhow::Result<cargo_metadata::Metadata> {
    let output = Command::from_command(cargo_metadata::MetadataCommand::new().cargo_command())
        .output()
        .context("exec")?;
    if !output.status.success() {
        anyhow::bail!("cargo metadata: {}", String::from_utf8(output.stderr)?)
    }
    let stdout = std::str::from_utf8(&output.stdout)?
        .lines()
        .find(|line| line.starts_with('{'))
        .context("metadata has no json")?;

    cargo_metadata::MetadataCommand::parse(stdout).context("parse")
}

pub fn cargo_cmd() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
}

pub fn comma_delimited(features: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut string = String::new();
    for feature in features {
        if !string.is_empty() {
            string.push(',');
        }
        string.push_str(feature.as_ref());
    }
    string
}

pub struct Command {
    command: std::process::Command,
}

impl Command {
    pub fn new(arg: impl AsRef<OsStr>) -> Self {
        Self {
            command: std::process::Command::new(arg),
        }
    }

    pub fn from_command(command: std::process::Command) -> Self {
        Self { command }
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.command.arg(arg);
        self
    }

    pub fn args(&mut self, arg: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        self.command.args(arg);
        self
    }

    pub fn env(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> &mut Self {
        self.command.env(key, val);
        self
    }

    pub fn env_clear(&mut self) -> &mut Self {
        self.command.env_clear();
        self
    }

    pub fn stdin(&mut self, stdin: impl Into<std::process::Stdio>) -> &mut Self {
        self.command.stdin(stdin);
        self
    }

    pub fn stdout(&mut self, stdin: impl Into<std::process::Stdio>) -> &mut Self {
        self.command.stdout(stdin);
        self
    }

    pub fn stderr(&mut self, stdin: impl Into<std::process::Stdio>) -> &mut Self {
        self.command.stderr(stdin);
        self
    }

    pub fn spawn(&mut self) -> std::io::Result<std::process::Child> {
        tracing::debug!("executing: {self}");
        self.command.spawn()
    }

    pub fn status(&mut self) -> std::io::Result<std::process::ExitStatus> {
        tracing::debug!("executing: {self}");
        self.command.status()
    }

    pub fn output(&mut self) -> std::io::Result<std::process::Output> {
        tracing::debug!("executing: {self}");
        self.command.output()
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = std::iter::once(self.command.get_program()).chain(self.command.get_args());
        for (idx, arg) in args.enumerate() {
            if idx > 0 {
                f.write_str(" ")?;
            }

            let arg = arg.to_string_lossy();
            let has_spaces = arg.split_whitespace().skip(1).next().is_some();
            if has_spaces {
                f.write_char('\'')?;
            }
            f.write_str(&arg)?;
            if has_spaces {
                f.write_char('\'')?;
            }
        }
        Ok(())
    }
}
