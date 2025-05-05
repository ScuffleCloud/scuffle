//! A crate designed to provide a simple interface to load and manage settings.
//!
//! This crate is a wrapper around the `config` crate and `clap` crate
//! to provide a simple interface to load and manage settings.
//!
//! ## How to use this
//!
//! ### With `scuffle_bootstrap`
//!
//! ```rust
//! // Define a config struct like this
//! // You can use all of the serde attributes to customize the deserialization
//! #[derive(serde_derive::Deserialize)]
//! struct MyConfig {
//!     some_setting: String,
//!     #[serde(default)]
//!     some_other_setting: i32,
//! }
//!
//! // Implement scuffle_boostrap::ConfigParser for the config struct like this
//! scuffle_settings::bootstrap!(MyConfig);
//!
//! # use std::sync::Arc;
//! /// Our global state
//! struct Global;
//!
//! impl scuffle_bootstrap::global::Global for Global {
//!     type Config = MyConfig;
//!
//!     async fn init(config: MyConfig) -> anyhow::Result<Arc<Self>> {
//!         // Here you now have access to the config
//!         Ok(Arc::new(Self))
//!     }
//! }
//! ```
//!
//! ### Without `scuffle_bootstrap`
//!
//! ```rust
//! # fn test() -> Result<(), scuffle_settings::SettingsError> {
//! // Define a config struct like this
//! // You can use all of the serde attributes to customize the deserialization
//! #[derive(serde_derive::Deserialize)]
//! struct MyConfig {
//!     some_setting: String,
//!     #[serde(default)]
//!     some_other_setting: i32,
//! }
//!
//! // Parsing options
//! let options = scuffle_settings::Options {
//!     env_prefix: Some("MY_APP"),
//!     ..Default::default()
//! };
//! // Parse the settings
//! let settings: MyConfig = scuffle_settings::parse_settings(options)?;
//! # Ok(())
//! # }
//! # unsafe { std::env::set_var("MY_APP_SOME_SETTING", "value"); }
//! # test().unwrap();
//! ```
//!
//! See [`Options`] for more information on how to customize parsing.
//!
//! ## Templates
//!
//! If the `templates` feature is enabled, the parser will attempt to render
//! the configuration file as a jinja template before processing it.
//!
//! All environment variables set during execution will be available under
//! the `env` variable inside the file.
//!
//! Example TOML file:
//!
//! ```toml
//! some_setting = "${{ env.MY_APP_SECRET }}"
//! ```
//!
//! Use `${{` and `}}` for variables, `{%` and `%}` for blocks and `{#` and `#}` for comments.
//!
//! ## Command Line Interface
//!
//! The following options are available for the CLI:
//!
//! - `--config` or `-c`
//!
//!   Path to a configuration file. This option can be used multiple times to load multiple files.
//! - `--override` or `-o`
//!
//!   Provide an override for a configuration value, in the format `KEY=VALUE`.
//!
//! ## Feature Flags
//!
//! - `full`: Enables all of the following features
//! - `templates`: Enables template support
//!
//!   See [Templates](#templates) above.
//! - `bootstrap`: Enables the `bootstrap!` macro
//!
//!   See [`bootstrap!`] and [With `scuffle_bootstrap`](#with-scuffle_bootstrap) above.
//! - `cli`: Enables the CLI
//!
//!   See [Command Line Interface](#command-line-interface) above.
//! - `all-formats`: Enables all of the following formats
//!
//! ### Format Feature Flags
//!
//! - `toml`: Enables TOML support
//! - `yaml`: Enables YAML support
//! - `json`: Enables JSON support
//! - `json5`: Enables JSON5 support
//! - `ron`: Enables RON support
//! - `ini`: Enables INI support
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
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]

use std::borrow::Cow;
use std::path::Path;

use config::FileStoredFormat;

mod options;

pub use options::*;

#[derive(Debug, Clone, Copy)]
struct FormatWrapper;

#[cfg(not(feature = "templates"))]
fn template_text<'a>(
    text: &'a str,
    _: &config::FileFormat,
) -> Result<Cow<'a, str>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Cow::Borrowed(text))
}

#[cfg(feature = "templates")]
fn template_text<'a>(
    text: &'a str,
    _: &config::FileFormat,
) -> Result<Cow<'a, str>, Box<dyn std::error::Error + Send + Sync>> {
    use minijinja::syntax::SyntaxConfig;

    let mut env = minijinja::Environment::new();

    env.add_global("env", std::env::vars().collect::<std::collections::HashMap<_, _>>());
    env.set_syntax(
        SyntaxConfig::builder()
            .block_delimiters("{%", "%}")
            .variable_delimiters("${{", "}}")
            .comment_delimiters("{#", "#}")
            .build()
            .unwrap(),
    );

    Ok(Cow::Owned(env.template_from_str(text).unwrap().render(())?))
}

impl config::Format for FormatWrapper {
    fn parse(
        &self,
        uri: Option<&String>,
        text: &str,
    ) -> Result<config::Map<String, config::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let uri_ext = uri.and_then(|s| Path::new(s.as_str()).extension()).and_then(|s| s.to_str());

        let mut formats: Vec<config::FileFormat> = vec![
            #[cfg(feature = "toml")]
            config::FileFormat::Toml,
            #[cfg(feature = "json")]
            config::FileFormat::Json,
            #[cfg(feature = "yaml")]
            config::FileFormat::Yaml,
            #[cfg(feature = "json5")]
            config::FileFormat::Json5,
            #[cfg(feature = "ini")]
            config::FileFormat::Ini,
            #[cfg(feature = "ron")]
            config::FileFormat::Ron,
        ];

        if let Some(uri_ext) = uri_ext {
            formats.sort_by_key(|f| if f.file_extensions().contains(&uri_ext) { 0 } else { 1 });
        }

        for format in formats {
            if let Ok(map) = format.parse(uri, template_text(text, &format)?.as_ref()) {
                return Ok(map);
            }
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("No supported format found for file: {uri:?}"),
        )))
    }
}

impl config::FileStoredFormat for FormatWrapper {
    fn file_extensions(&self) -> &'static [&'static str] {
        &[
            #[cfg(feature = "toml")]
            "toml",
            #[cfg(feature = "json")]
            "json",
            #[cfg(feature = "yaml")]
            "yaml",
            #[cfg(feature = "yaml")]
            "yml",
            #[cfg(feature = "json5")]
            "json5",
            #[cfg(feature = "ini")]
            "ini",
            #[cfg(feature = "ron")]
            "ron",
        ]
    }
}

/// An error that can occur when parsing settings.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// An error occurred while parsing the settings.
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    /// An error occurred while parsing the CLI arguments.
    #[cfg(feature = "cli")]
    #[error(transparent)]
    Clap(#[from] clap::Error),
}

/// Parse settings using the given options.
///
/// Refer to the [`Options`] struct for more information on how to customize parsing.
pub fn parse_settings<T: serde::de::DeserializeOwned>(options: Options) -> Result<T, SettingsError> {
    let mut config = config::Config::builder();

    #[allow(unused_mut)]
    let mut added_files = false;

    #[cfg(feature = "cli")]
    if let Some(cli) = options.cli {
        let command = clap::Command::new(cli.name)
            .version(cli.version)
            .about(cli.about)
            .author(cli.author)
            .bin_name(cli.name)
            .arg(
                clap::Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Path to configuration file(s)")
                    .action(clap::ArgAction::Append),
            )
            .arg(
                clap::Arg::new("overrides")
                    .long("override")
                    .short('o')
                    .alias("set")
                    .help("Provide an override for a configuration value, in the format KEY=VALUE")
                    .action(clap::ArgAction::Append),
            );

        let matches = command.get_matches_from(cli.argv);

        if let Some(config_files) = matches.get_many::<String>("config") {
            for path in config_files {
                config = config.add_source(config::File::new(path, FormatWrapper));
                added_files = true;
            }
        }

        if let Some(overrides) = matches.get_many::<String>("overrides") {
            for ov in overrides {
                let (key, value) = ov.split_once('=').ok_or_else(|| {
                    clap::Error::raw(
                        clap::error::ErrorKind::InvalidValue,
                        "Override must be in the format KEY=VALUE",
                    )
                })?;

                config = config.set_override(key, value)?;
            }
        }
    }

    if !added_files {
        if let Some(default_config_file) = options.default_config_file {
            config = config.add_source(config::File::new(default_config_file, FormatWrapper).required(false));
        }
    }

    if let Some(env_prefix) = options.env_prefix {
        config = config.add_source(config::Environment::with_prefix(env_prefix));
    }

    Ok(config.build()?.try_deserialize()?)
}

#[doc(hidden)]
#[cfg(feature = "bootstrap")]
pub mod macros {
    pub use {anyhow, scuffle_bootstrap};
}

/// This macro can be used to integrate with the [`scuffle_bootstrap`] ecosystem.
///
/// This macro will implement the [`scuffle_bootstrap::config::ConfigParser`] trait for the given type.
/// The generated implementation uses the [`parse_settings`] function to parse the settings.
///
/// ## Example
///
/// ```rust
/// #[derive(serde_derive::Deserialize)]
/// struct MySettings {
///     key: String,
/// }
/// ```
#[cfg(feature = "bootstrap")]
#[macro_export]
macro_rules! bootstrap {
    ($ty:ty) => {
        impl $crate::macros::scuffle_bootstrap::config::ConfigParser for $ty {
            async fn parse() -> $crate::macros::anyhow::Result<Self> {
                $crate::macros::anyhow::Context::context(
                    $crate::parse_settings($crate::Options {
                        cli: Some($crate::cli!()),
                        ..::std::default::Default::default()
                    }),
                    "config",
                )
            }
        }
    };
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use serde_derive::Deserialize;

    #[cfg(feature = "cli")]
    use crate::Cli;
    use crate::{Options, parse_settings};

    #[derive(Debug, Deserialize)]
    struct TestSettings {
        #[cfg_attr(not(feature = "cli"), allow(dead_code))]
        key: String,
    }

    #[test]
    fn parse_empty() {
        let err = parse_settings::<TestSettings>(Options::default()).expect_err("expected error");
        assert!(matches!(err, crate::SettingsError::Config(config::ConfigError::Message(_))));
        assert_eq!(err.to_string(), "missing field `key`");
    }

    #[test]
    #[cfg(feature = "cli")]
    fn parse_cli() {
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec!["test".to_string(), "-o".to_string(), "key=value".to_string()],
            }),
            ..Default::default()
        };
        let settings: TestSettings = parse_settings(options).expect("failed to parse settings");

        assert_eq!(settings.key, "value");
    }

    #[test]
    #[cfg(feature = "cli")]
    fn cli_error() {
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec!["test".to_string(), "-o".to_string(), "error".to_string()],
            }),
            ..Default::default()
        };
        let err = parse_settings::<TestSettings>(options).expect_err("expected error");

        if let crate::SettingsError::Clap(err) = err {
            assert_eq!(err.to_string(), "error: Override must be in the format KEY=VALUE");
        } else {
            panic!("unexpected error: {err}");
        }
    }

    #[test]
    #[cfg(all(feature = "cli", feature = "toml"))]
    fn parse_file() {
        use std::path::Path;

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("test.toml");
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec!["test".to_string(), "-c".to_string(), path.display().to_string()],
            }),
            ..Default::default()
        };
        let settings: TestSettings = parse_settings(options).expect("failed to parse settings");

        assert_eq!(settings.key, "filevalue");
    }

    #[test]
    #[cfg(feature = "cli")]
    fn file_error() {
        use std::path::Path;

        let path = Path::new("assets").join("invalid.txt");
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec!["test".to_string(), "-c".to_string(), path.display().to_string()],
            }),
            ..Default::default()
        };
        let err = parse_settings::<TestSettings>(options).expect_err("expected error");

        if let crate::SettingsError::Config(config::ConfigError::FileParse { uri: Some(uri), cause }) = err {
            assert_eq!(uri, path.display().to_string());
            assert_eq!(
                cause.to_string(),
                format!("No supported format found for file: {:?}", path.to_str())
            );
        } else {
            panic!("unexpected error: {err:?}");
        }
    }

    #[test]
    #[cfg(feature = "cli")]
    fn parse_env() {
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec![],
            }),
            env_prefix: Some("SETTINGS_PARSE_ENV_TEST"),
            ..Default::default()
        };
        // Safety: This is a test and we do not have multiple threads.
        unsafe {
            std::env::set_var("SETTINGS_PARSE_ENV_TEST_KEY", "envvalue");
        }
        let settings: TestSettings = parse_settings(options).expect("failed to parse settings");

        assert_eq!(settings.key, "envvalue");
    }

    #[test]
    #[cfg(feature = "cli")]
    fn overrides() {
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec!["test".to_string(), "-o".to_string(), "key=value".to_string()],
            }),
            env_prefix: Some("SETTINGS_OVERRIDES_TEST"),
            ..Default::default()
        };
        // Safety: This is a test and we do not have multiple threads.
        unsafe {
            std::env::set_var("SETTINGS_OVERRIDES_TEST_KEY", "envvalue");
        }
        let settings: TestSettings = parse_settings(options).expect("failed to parse settings");

        assert_eq!(settings.key, "value");
    }

    #[test]
    #[cfg(all(feature = "templates", feature = "cli"))]
    fn templates() {
        let options = Options {
            cli: Some(Cli {
                name: "test",
                version: "0.1.0",
                about: "test",
                author: "test",
                argv: vec!["test".to_string(), "-c".to_string(), "assets/templates.toml".to_string()],
            }),
            ..Default::default()
        };
        // Safety: This is a test and we do not have multiple threads.
        unsafe {
            std::env::set_var("SETTINGS_TEMPLATES_TEST", "templatevalue");
        }
        let settings: TestSettings = parse_settings(options).expect("failed to parse settings");

        assert_eq!(settings.key, "templatevalue");
    }
}
