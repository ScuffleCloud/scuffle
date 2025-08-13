use std::ffi::OsStr;
use std::fmt::Write;
use std::process::Stdio;
use std::sync::{Condvar, Mutex};

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

pub fn metadata() -> anyhow::Result<cargo_metadata::Metadata> {
    metadata_for_manifest(None)
}

pub fn metadata_for_manifest(manifest: Option<&Utf8Path>) -> anyhow::Result<cargo_metadata::Metadata> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(manifest) = manifest {
        cmd.manifest_path(manifest);
    }
    let output = Command::from_command(cmd.cargo_command()).output().context("exec")?;
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
            let has_spaces = arg.split_whitespace().nth(1).is_some();
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

pub fn git_workdir_clean() -> anyhow::Result<()> {
    const ERROR_MESSAGE: &str = "git working directory is dirty, please commit your changes or run with --allow-dirty";
    anyhow::ensure!(
        Command::new("git")
            .arg("diff")
            .arg("--exit-code")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .context("git diff")?
            .status
            .success(),
        ERROR_MESSAGE,
    );

    anyhow::ensure!(
        Command::new("git")
            .arg("diff")
            .arg("--staged")
            .arg("--exit-code")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .context("git diff")?
            .status
            .success(),
        ERROR_MESSAGE,
    );

    Ok(())
}

struct Semaphore {
    count: Mutex<usize>,
    cvar: Condvar,
}

impl Semaphore {
    fn new(initial: usize) -> Self {
        Self {
            count: Mutex::new(if initial == 0 { usize::MAX } else { initial }),
            cvar: Condvar::new(),
        }
    }

    fn acquire(&self) {
        let count = self.count.lock().unwrap();
        let mut count = self.cvar.wait_while(count, |count| *count == 0).unwrap();
        *count -= 1;
    }

    fn release(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        self.cvar.notify_one();
    }
}

pub fn concurrently<U: Send, T: Send, C: FromIterator<T>>(
    concurrency: usize,
    items: impl IntoIterator<Item = U>,
    func: impl Fn(U) -> T + Send + Sync,
) -> C {
    let sem = Semaphore::new(concurrency);
    std::thread::scope(|s| {
        let items = items
            .into_iter()
            .map(|item| {
                s.spawn(|| {
                    sem.acquire();
                    let r = func(item);
                    sem.release();
                    r
                })
            })
            .collect::<Vec<_>>();
        C::from_iter(items.into_iter().map(|item| item.join().unwrap()))
    })
}

pub fn relative_to(path: &Utf8Path, dir: &Utf8Path) -> Utf8PathBuf {
    // If the path is already relative, just return it as is
    if path.is_relative() {
        return path.to_owned();
    }

    // Attempt to strip the prefix
    if let Ok(stripped) = path.strip_prefix(dir) {
        return stripped.to_owned();
    }

    // Fall back to manual computation like pathdiff does
    let mut result = Utf8PathBuf::new();

    let mut dir_iter = dir.components();
    let mut path_iter = path.components();

    // Skip common prefix components
    while let (Some(d), Some(p)) = (dir_iter.clone().next(), path_iter.clone().next()) {
        if d == p {
            dir_iter.next();
            path_iter.next();
        } else {
            break;
        }
    }

    // For remaining components in dir, add ".."
    for _ in dir_iter {
        result.push("..");
    }

    // Add remaining components from path
    for p in path_iter {
        result.push(p);
    }

    result
}

pub struct DropRunner<F: FnOnce()> {
    func: Option<F>,
}

impl<F: FnOnce()> DropRunner<F> {
    pub fn new(func: F) -> Self {
        Self { func: Some(func) }
    }
}

impl<F: FnOnce()> Drop for DropRunner<F> {
    fn drop(&mut self) {
        if let Some(func) = self.func.take() {
            func()
        }
    }
}
