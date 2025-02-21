//! A crate which allows you to compile Rust code at runtime (hence the name
//! `postcompile`).
//!
//! What that means is that you can provide the input to `rustc` and then get
//! back the expanded output, compiler errors, warnings, etc.
//!
//! This is particularly useful when making snapshot tests of proc-macros, look
//! below for an example with the `insta` crate.
//!
//! ## Usage
//!
//! ```rs
//! #[test]
//! fn some_cool_test() {
//!     insta::assert_snapshot!(postcompile::compile! {
//!         #![allow(unused)]
//!
//!         #[derive(Debug, Clone)]
//!         struct Test {
//!             a: u32,
//!             b: i32,
//!         }
//!
//!         const TEST: Test = Test { a: 1, b: 3 };
//!     });
//! }
//!
//! #[test]
//! fn some_cool_test_extern() {
//!     insta::assert_snapshot!(postcompile::compile_str!(include_str!("some_file.rs")));
//! }
//! ```
//!
//! ## Features
//!
//! - Cached builds: This crate reuses the cargo build cache of the original
//!   crate so that only the contents of the macro are compiled & not any
//!   additional dependencies.
//! - Coverage: This crate works with [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov)
//!   out of the box, which allows you to instrument the proc-macro expansion.
//!
//! ## Alternatives
//!
//! - [`compiletest_rs`](https://crates.io/crates/compiletest_rs): This crate is
//!   used by the Rust compiler team to test the compiler itself. Not really
//!   useful for proc-macros.
//! - [`trybuild`](https://crates.io/crates/trybuild): This crate is an
//!   all-in-one solution for testing proc-macros, with built in snapshot
//!   testing.
//! - [`ui_test`](https://crates.io/crates/ui_test): Similar to `trybuild` with
//!   a slightly different API & used by the Rust compiler team to test the
//!   compiler itself.
//!
//! ### Differences
//!
//! The other libraries are focused on testing & have built in test harnesses.
//! This crate takes a step back and allows you to compile without a testing
//! harness. This has the advantage of being more flexible, and allows you to
//! use whatever testing framework you want.
//!
//! In the examples above I showcase how to use this crate with the `insta`
//! crate for snapshot testing.
//!
//! ## Status
//!
//! This crate is currently under development and is not yet stable.
//!
//! Unit tests are not yet fully implemented. Use at your own risk.
//!
//! ## Limitations
//!
//! Please note that this crate does not work inside a running compiler process
//! (inside a proc-macro) without hacky workarounds and complete build-cache
//! invalidation.
//!
//! This is because `cargo` holds a lock on the build directory and that if we
//! were to compile inside a proc-macro we would recursively invoke the
//! compiler.
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;

use deps::{Dependencies, Errored};

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
        writeln!(f, "exit status: {}", self.status)?;
        if !self.stderr.is_empty() {
            write!(f, "--- stderr \n{}\n", self.stderr)?;
        }
        if !self.stdout.is_empty() {
            write!(f, "--- stdout \n{}\n", self.stdout)?;
        }
        Ok(())
    }
}

fn rustc(config: &Config, tmp_file: &Path) -> Command {
    let mut program = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    program.env("RUSTC_BOOTSTRAP", "1");
    let rust_flags = std::env::var_os("RUSTFLAGS");

    if let Some(rust_flags) = &rust_flags {
        program.args(
            rust_flags
                .as_encoded_bytes()
                .split(|&b| b == b' ')
                .map(|flag| OsString::from(OsStr::from_bytes(flag))),
        );
    }

    program.arg("--crate-name");
    program.arg(config.function_name.split("::").last().unwrap_or("unnamed"));
    program.arg(tmp_file);
    program.envs(std::env::vars());

    program.stderr(std::process::Stdio::piped());
    program.stdout(std::process::Stdio::piped());

    program
}

fn write_tmp_file(tokens: &str, tmp_file: &Path) {
    #[cfg(feature = "prettyplease")]
    {
        if let Ok(syn_file) = syn::parse_file(tokens) {
            let pretty_file = prettyplease::unparse(&syn_file);
            std::fs::write(tmp_file, pretty_file).unwrap();
            return;
        }
    }

    std::fs::write(tmp_file, tokens).unwrap();
}

/// Compiles the given tokens and returns the output.
pub fn compile_custom(tokens: &str, config: &Config) -> Result<CompileOutput, Errored> {
    let tmp_file = Path::new(config.tmp_dir.as_ref()).join(format!("{}.rs", config.function_name));

    write_tmp_file(tokens, &tmp_file);

    let dependencies = Dependencies::new(config)?;

    let mut program = rustc(config, &tmp_file);

    dependencies.apply(&mut program);
    // The first invoke is used to get the macro expanded code.
    program.arg("-Zunpretty=expanded");

    let output = program.output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let syn_file = syn::parse_file(&stdout);
    #[cfg(feature = "prettyplease")]
    let stdout = syn_file.as_ref().map(prettyplease::unparse).unwrap_or(stdout);

    let mut crate_type = "lib";

    if let Ok(file) = syn_file {
        if file.items.iter().any(|item| {
            let syn::Item::Fn(func) = item else {
                return false;
            };

            func.sig.ident == "main"
        }) {
            crate_type = "bin";
        }
    };

    let mut status = if output.status.success() {
        ExitStatus::Success
    } else {
        ExitStatus::Failure(output.status.code().unwrap_or(-1))
    };

    let stderr = if status == ExitStatus::Success {
        let mut program = rustc(config, &tmp_file);
        dependencies.apply(&mut program);
        program.arg("--emit=llvm-ir");
        program.arg(format!("--crate-type={crate_type}"));
        program.arg("-o");
        program.arg("-");
        let comp_output = program.output().unwrap();
        status = if comp_output.status.success() {
            ExitStatus::Success
        } else {
            ExitStatus::Failure(comp_output.status.code().unwrap_or(-1))
        };
        String::from_utf8(comp_output.stderr).unwrap()
    } else {
        String::from_utf8(output.stderr).unwrap()
    };

    let stderr = stderr.replace(tmp_file.as_os_str().to_string_lossy().as_ref(), "<postcompile>");
    let stdout = stdout.replace(tmp_file.as_os_str().to_string_lossy().as_ref(), "<postcompile>");

    Ok(CompileOutput { status, stdout, stderr })
}

/// The configuration for the compilation.
#[derive(Clone, Debug)]
pub struct Config {
    /// The path to the cargo manifest file of the library being tested.
    /// This is so that we can include the `dependencies` & `dev-dependencies`
    /// making them available in the code provided.
    pub manifest: Cow<'static, Path>,
    /// The path to the target directory, used to cache builds & find
    /// dependencies.
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
/// This macro will return an error if we fail to invoke the compiler. Unlike
/// the [`compile!`] macro, this will not panic.
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
/// Same as the [`try_compile!`] macro, but for strings similar usage to
/// [`compile_str!`].
#[macro_export]
macro_rules! try_compile_str {
    ($expr:expr) => {
        $crate::compile_custom($expr, &$crate::_config!())
    };
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use insta::assert_snapshot;

    use crate::ExitStatus;

    #[test]
    fn compile_success() {
        let out = compile! {
            #[allow(unused)]
            fn main() {
                let a = 1;
                let b = 2;
                let c = a + b;
            }
        };

        assert_eq!(out.status, ExitStatus::Success);
        assert!(out.stderr.is_empty());
        assert_snapshot!(out);
    }

    #[test]
    fn try_compile_success() {
        let out = try_compile! {
            #[allow(unused)]
            fn main() {
                let xd = 0xd;
                let xdd = 0xdd;
                let xddd = xd + xdd;
                println!("{}", xddd);
            }
        };

        assert!(out.is_ok());
        let out = out.unwrap();
        assert_eq!(out.status, ExitStatus::Success);
        assert!(out.stderr.is_empty());
        assert!(!out.stdout.is_empty());
    }

    #[test]
    fn compile_failure() {
        let out = compile! {
            invalid_rust_code
        };

        assert_eq!(out.status, ExitStatus::Failure(1));
        assert!(out.stdout.is_empty());
        assert_snapshot!(out);
    }

    #[test]
    fn try_compile_failure() {
        let out = try_compile! {
            invalid rust code
        };

        assert!(out.is_ok());
        let out = out.unwrap();
        assert_eq!(out.status, ExitStatus::Failure(1));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
}
