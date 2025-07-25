<!-- sync-readme title [[ -->
# scuffle-settings
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-settings/0.1.4.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-settings/0.1.4)
[![crates.io](https://img.shields.io/badge/crates.io-v0.1.4-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-settings/0.1.4)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-settings/0.1.4.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-settings/0.1.4.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A crate designed to provide a simple interface to load and manage settings.

This crate is a wrapper around the `config` crate and `clap` crate
to provide a simple interface to load and manage settings.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`cli`** —  Enables cli parsing using clap
* **`ron`** —  Enables the ron format
* **`toml`** —  Enables the toml format
* **`yaml`** —  Enables the yaml format
* **`json`** —  Enables the json format
* **`json5`** —  Enables the json5 formast
* **`ini`** —  Enables the ini format
* **`all-formats`** —  Enables all formats
* **`templates`** —  Enables templating support via jinja
* **`bootstrap`** —  Enables scuffle-bootstrap support
* **`full`** —  Enables everything
* **`docs`** —  Enables changelog and documentation of feature flags

### Examples

#### With [`scuffle_bootstrap`](https://docs.rs/scuffle-bootstrap/latest)

````rust
// Define a config struct like this
// You can use all of the serde attributes to customize the deserialization
#[derive(serde_derive::Deserialize)]
struct MyConfig {
    some_setting: String,
    #[serde(default)]
    some_other_setting: i32,
}

// Implement scuffle_boostrap::ConfigParser for the config struct like this
scuffle_settings::bootstrap!(MyConfig);

/// Our global state
struct Global;

impl scuffle_bootstrap::global::Global for Global {
    type Config = MyConfig;

    async fn init(config: MyConfig) -> anyhow::Result<Arc<Self>> {
        // Here you now have access to the config
        Ok(Arc::new(Self))
    }
}
````

#### Without `scuffle_bootstrap`

````rust
// Define a config struct like this
// You can use all of the serde attributes to customize the deserialization
#[derive(serde_derive::Deserialize)]
struct MyConfig {
    some_setting: String,
    #[serde(default)]
    some_other_setting: i32,
}

// Parsing options
let options = scuffle_settings::Options {
    env_prefix: Some("MY_APP"),
    ..Default::default()
};
// Parse the settings
let settings: MyConfig = scuffle_settings::parse_settings(options)?;
````

See [`Options`](https://docs.rs/scuffle-settings/0.1.4/scuffle_settings/options/struct.Options.html) for more information on how to customize parsing.

### Templates

If the `templates` feature is enabled, the parser will attempt to render
the configuration file as a jinja template before processing it.

All environment variables set during execution will be available under
the `env` variable inside the file.

Example TOML file:

````toml
some_setting = "${{ env.MY_APP_SECRET }}"
````

Use `${{` and `}}` for variables, `{%` and `%}` for blocks and `{#` and `#}` for comments.

### Command Line Interface

The following options are available for the CLI:

* `--config` or `-c`
  
  Path to a configuration file. This option can be used multiple times to load multiple files.

* `--override` or `-o`
  
  Provide an override for a configuration value, in the format `KEY=VALUE`.

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
