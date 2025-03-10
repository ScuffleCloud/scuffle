use anyhow::Context;
use extensions::Extensions;
use prost_reflect::DescriptorPool;

mod extensions;

#[derive(Debug, Default)]
pub struct Config {
    tonic: tonic_build::Config,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: tonic_build::Config) -> Self {
        self.tonic = config;
        self
    }

    pub fn generate(&mut self, protos: &[&str], includes: &[&str]) -> anyhow::Result<()> {
        let out_dir = std::path::PathBuf::from(
            std::env::var("OUT_DIR").context("OUT_DIR must be set, typically set by a cargo build script")?,
        );

        let ft_path = out_dir.join("tinc.fd.bin");
        self.tonic.file_descriptor_set_path(&ft_path);

        let fds = self
            .tonic
            .load_fds(protos, includes)
            .context("failed to generate tonic fds")?;

        let fds_bytes = std::fs::read(ft_path).context("failed to read tonic fds")?;

        let pool = DescriptorPool::decode(&mut fds_bytes.as_slice()).context("failed to decode tonic fds")?;

        let mut extensions = Extensions::new(&pool);

        extensions.process(&pool).context("failed to process extensions")?;

        for (key, message) in extensions.messages() {
            if message.opts.skip_derive {
                continue;
            }

            self.tonic.message_attribute(key, "#[derive(::tinc::reexports::serde::Serialize, ::tinc::reexports::serde::Deserialize, ::tinc::reexports::schemars::JsonSchema)]");
            self.tonic
                .message_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
            self.tonic
                .message_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
            self.tonic.message_attribute(key, "#[serde(default)]");
            for (field, field_opts) in &message.fields {
                let name = field_opts.opts.rename.as_ref().unwrap_or(&field_opts.json_name);
                self.tonic
                    .field_attribute(format!("{}.{}", key, field), format!("#[serde(rename = \"{}\")]", name));
                if let Some(serde_with) = field_opts.kind.serde_with(key) {
                    self.tonic
                        .field_attribute(format!("{}.{}", key, field), format!("#[serde(with = \"{serde_with}\")]"));
                }
                if let Some(schemars_with) = field_opts.kind.schemars_with(key) {
                    self.tonic.field_attribute(
                        format!("{}.{}", key, field),
                        format!("#[schemars(with = \"{schemars_with}\")]"),
                    );
                }
            }
        }

        for (key, enum_) in extensions.enums() {
            if enum_.opts.skip_derive {
                continue;
            }

            self.tonic.enum_attribute(key, "#[derive(::tinc::reexports::serde::Serialize, ::tinc::reexports::serde::Deserialize, ::tinc::reexports::schemars::JsonSchema)]");
            self.tonic
                .enum_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
            self.tonic
                .enum_attribute(key, "#[schemars(crate = \"::tinc::reexports::schemars\")]");
            for (variant, variant_opts) in &enum_.variants {
                if let Some(rename) = &variant_opts.opts.rename {
                    self.tonic
                        .field_attribute(format!("{}.{}", key, variant), format!("#[serde(rename = \"{}\")]", rename));
                }
            }
        }
        dbg!(&extensions.messages());
        dbg!(&extensions.enums());
        dbg!(&extensions.services());

        self.tonic.compile_fds(fds).context("failed to compile tonic fds")?;

        Ok(())
    }
}
