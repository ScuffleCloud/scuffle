#![doc = include_str!("../README.md")]

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::Path,
    process::Command,
};

use deps::Errored;

mod deps;
mod features;

/// The return status of the compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// If the compiler returned a 0 exit code.
    Success,
    /// If the compiler returned a non-0 exit code.
    Failure(i32),
}

impl std::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitStatus::Success => write!(f, "0"),
            ExitStatus::Failure(code) => write!(f, "{}", code),
        }
    }
}

/// The output of the compilation.
#[derive(Debug)]
pub struct CompileOutput {
    /// The status of the compilation.
    pub status: ExitStatus,
    /// The stdout of the compilation.
    /// This will contain the expanded code.
    pub stdout: String,
    /// The stderr of the compilation.
    /// This will contain any errors or warnings from the compiler.
    pub stderr: String,
}

impl std::fmt::Display for CompileOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit status: {}\n", self.status)?;
        if !self.stderr.is_empty() {
            write!(f, "--- stderr \n{}\n", self.stderr)?;
        }
        if !self.stdout.is_empty() {
            write!(f, "--- stdout \n{}\n", self.stdout)?;
        }
        Ok(())
    }
}

/// Compiles the given tokens and returns the output.
pub fn compile_custom(tokens: &[u8], config: &Config) -> Result<CompileOutput, Errored> {
    let mut program = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    program.env("RUSTC_BOOTSTRAP", "1");
    program.arg("-Zunpretty=expanded");

    let rust_flags = std::env::var_os("RUSTFLAGS");

    if let Some(rust_flags) = &rust_flags {
        program.args(
            rust_flags
                .as_encoded_bytes()
                .split(|&b| b == b' ')
                .map(|flag| OsString::from(OsStr::from_bytes(flag))),
        );
    }

    program.arg("--out-dir");
    program.arg(config.tmp_dir.as_ref());
    program.arg("--crate-name");
    program.arg(config.function_name.split("::").last().unwrap_or("unnamed"));

    let tmp_file = Path::new(config.tmp_dir.as_ref()).join(format!("{}.rs", config.function_name));

    std::fs::write(&tmp_file, tokens).unwrap();

    program.arg(&tmp_file);
    program.envs(std::env::vars());

    deps::build_dependencies(config, &mut program)?;

    program.stderr(std::process::Stdio::piped());
    program.stdout(std::process::Stdio::piped());

    let child = program.spawn().unwrap();

    let output = child.wait_with_output().unwrap();

    Ok(CompileOutput {
        status: if output.status.success() {
            ExitStatus::Success
        } else {
            ExitStatus::Failure(output.status.code().unwrap_or(-1))
        },
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    })
}

/// The configuration for the compilation.
#[derive(Clone, Debug)]
pub struct Config {
    /// The path to the cargo manifest file of the library being tested.
    /// This is so that we can include the `dependencies` & `dev-dependencies` making them available in the code provided.
    pub manifest: Cow<'static, Path>,
    /// The path to the target directory, used to cache builds & find dependencies.
    pub target_dir: Cow<'static, Path>,
    /// A temporary directory to write the expanded code to.
    pub tmp_dir: Cow<'static, Path>,
    /// The name of the function to compile.
    pub function_name: Cow<'static, str>,
}

#[macro_export]
#[doc(hidden)]
macro_rules! _function_name {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of_val(f).strip_suffix("::f").unwrap_or("");
        while let Some(rest) = name.strip_suffix("::{{closure}}") {
            name = rest;
        }
        name
    }};
}

#[doc(hidden)]
pub fn build_dir() -> &'static Path {
    Path::new(env!("OUT_DIR"))
}

#[doc(hidden)]
pub fn target_dir() -> &'static Path {
    build_dir()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

#[macro_export]
#[doc(hidden)]
macro_rules! _config {
    () => {{
        $crate::Config {
            manifest: ::std::borrow::Cow::Borrowed(::std::path::Path::new(env!("CARGO_MANIFEST_PATH"))),
            tmp_dir: ::std::borrow::Cow::Borrowed($crate::build_dir()),
            target_dir: ::std::borrow::Cow::Borrowed($crate::target_dir()),
            function_name: ::std::borrow::Cow::Borrowed($crate::_function_name!()),
        }
    }};
}

/// Compiles the given tokens and returns the output.
///
/// This macro will panic if we fail to invoke the compiler.
///
/// ```rs
/// // Dummy macro to assert the snapshot.
/// macro_rules! assert_snapshot {
///     ($expr:expr) => {};
/// }
///
/// let output = postcompile::compile! {
///     const TEST: u32 = 1;
/// };
///
/// assert_eq!(output.status, postcompile::ExitStatus::Success);
/// assert!(output.stderr.is_empty());
/// assert_snapshot!(output.stdout); // We dont have an assert_snapshot! macro in this crate, but you get the idea.
/// ```
#[macro_export]
macro_rules! compile {
    ($($tokens:tt)*) => {
        $crate::compile_str!(stringify!($($tokens)*))
    };
}

/// Compiles the given string of tokens and returns the output.
///
/// This macro will panic if we fail to invoke the compiler.
///
/// Same as the [`compile!`] macro, but for strings. This allows you to do:
///
/// ```rs
/// let output = postcompile::compile_str!(include_str!("some_file.rs"));
///
/// // ... do something with the output
/// ```
#[macro_export]
macro_rules! compile_str {
    ($expr:expr) => {
        $crate::try_compile_str!($expr).expect("failed to compile")
    };
}

/// Compiles the given string of tokens and returns the output.
///
/// This macro will return an error if we fail to invoke the compiler. Unlike the [`compile!`] macro, this will not panic.
///
/// ```rs
/// let output = postcompile::try_compile! {
///     const TEST: u32 = 1;
/// };
///
/// assert!(output.is_ok());
/// assert_eq!(output.unwrap().status, postcompile::ExitStatus::Success);
/// ```
#[macro_export]
macro_rules! try_compile {
    ($($tokens:tt)*) => {
        $crate::try_compile_str!(stringify!($($tokens)*))
    };
}

/// Compiles the given string of tokens and returns the output.
///
/// This macro will return an error if we fail to invoke the compiler.
///
/// Same as the [`try_compile!`] macro, but for strings similar usage to [`compile_str!`].
#[macro_export]
macro_rules! try_compile_str {
    ($expr:expr) => {
        $crate::compile_custom($expr.as_bytes(), &$crate::_config!())
    };
}