use anyhow::Context;
use extensions::Extensions;
use prost_reflect::DescriptorPool;

mod extensions;

#[derive(Debug)]
pub struct Config {
    tonic: tonic_build::Builder,
    prost: tonic_build::Config,
    disable_tinc_include: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            tonic: tonic_build::configure(),
            disable_tinc_include: false,
            prost: tonic_build::Config::new(),
        }
    }

    pub fn with_tonic(mut self, config: tonic_build::Builder) -> Self {
        self.tonic = config;
        self
    }

    pub fn with_prost(mut self, config: tonic_build::Config) -> Self {
        self.prost = config;
        self
    }

    pub fn disable_tinc_include(mut self) -> Self {
        self.disable_tinc_include = true;
        self
    }

    pub fn compile_protos(mut self, protos: &[&str], includes: &[&str]) -> anyhow::Result<()> {
        let out_dir_str = std::env::var("OUT_DIR").context("OUT_DIR must be set, typically set by a cargo build script")?;
        let out_dir = std::path::PathBuf::from(&out_dir_str);

        let ft_path = out_dir.join("tinc.fd.bin");
        self.prost.file_descriptor_set_path(&ft_path);

        let mut includes = includes.to_vec();

        if !self.disable_tinc_include {
            let extra_includes = out_dir.join("tinc");
            self.prost.extern_path(".tinc", "::tinc::reexports::tinc_pb");
            std::fs::create_dir_all(&extra_includes).context("failed to create tinc directory")?;
            std::fs::write(extra_includes.join("annotations.proto"), tinc_pb::TINC_ANNOTATIONS)
                .context("failed to write tinc_annotations.rs")?;
            includes.push(&out_dir_str);
        }

        let fds = self
            .prost
            .load_fds(protos, &includes)
            .context("failed to generate tonic fds")?;

        let fds_bytes = std::fs::read(ft_path).context("failed to read tonic fds")?;

        let pool = DescriptorPool::decode(&mut fds_bytes.as_slice()).context("failed to decode tonic fds")?;

        let mut extensions = Extensions::new(&pool);

        extensions.process(&pool).context("failed to process extensions")?;

        for (key, message) in extensions.messages() {
            let message_custom_impl = message.opts.custom_impl.unwrap_or(false);
            for (oneof, oneof_opts) in &message.oneofs {
                let oneof_key = format!("{key}.{oneof}");

                if !message_custom_impl {
                    if !oneof_opts.opts.no_flatten.unwrap_or(false) {
                        self.prost.field_attribute(&oneof_key, "#[serde(flatten)]");
                    }

                    if let Some(rename) = &oneof_opts.opts.rename {
                        self.prost
                            .field_attribute(&oneof_key, format!("#[serde(rename = \"{rename}\")]"));
                    }
                }

                if oneof_opts.opts.custom_impl.unwrap_or(message_custom_impl) {
                    continue;
                }

                self.prost
                    .enum_attribute(&oneof_key, "#[derive(::tinc::reexports::serde::Serialize)]");
                self.prost
                    .enum_attribute(&oneof_key, "#[derive(::tinc::reexports::serde::Deserialize)]");
                self.prost
                    .enum_attribute(&oneof_key, "#[serde(crate = \"::tinc::reexports::serde\")]");
            }

            if message_custom_impl {
                continue;
            }

            self.prost
                .message_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
            self.prost
                .message_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
            self.prost
                .message_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
            self.prost.message_attribute(key, "#[serde(default)]");
            for (field, field_opts) in &message.fields {
                if field_opts
                    .one_of
                    .as_ref()
                    .is_some_and(|oneof| message.oneofs.get(oneof).unwrap().opts.custom_impl.unwrap_or(false))
                {
                    continue;
                }

                let name = field_opts.opts.rename.as_ref().unwrap_or(&field_opts.json_name);
                let field_key = if let Some(oneof) = &field_opts.one_of {
                    format!("{key}.{oneof}.{field}")
                } else {
                    format!("{key}.{field}")
                };

                self.prost
                    .field_attribute(&field_key, format!("#[serde(rename = \"{name}\")]"));
                if let Some(serde_with) = field_opts.kind.serde_with(key) {
                    self.prost
                        .field_attribute(&field_key, format!("#[serde(with = \"{serde_with}\")]"));
                }
            }
        }

        for (key, enum_) in extensions.enums() {
            if enum_.opts.custom_impl.unwrap_or(false) {
                continue;
            }

            self.prost
                .enum_attribute(key, "#[derive(::tinc::reexports::serde::Serialize)]");
            self.prost
                .enum_attribute(key, "#[derive(::tinc::reexports::serde::Deserialize)]");
            self.prost
                .enum_attribute(key, "#[serde(crate = \"::tinc::reexports::serde\")]");
            for (variant, variant_opts) in &enum_.variants {
                if let Some(rename) = &variant_opts.opts.rename {
                    self.prost
                        .field_attribute(format!("{}.{}", key, variant), format!("#[serde(rename = \"{}\")]", rename));
                }
            }
        }

        for file in &fds.file {
            dbg!(&file);
        }

        self.tonic.compile_fds_with_config(self.prost, fds).context("failed to compile tonic fds")?;

        Ok(())
    }
}
