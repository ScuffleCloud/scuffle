use anyhow::Context;
use extensions::Extensions;
use prost_reflect::DescriptorPool;

mod codegen;
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

        let modules = codegen::generate_modules(&extensions, &mut self.prost)?;

        self.tonic
            .compile_fds_with_config(self.prost, fds)
            .context("failed to compile tonic fds")?;

        for (package, module) in modules {
            let path = out_dir.join(format!("{package}.rs"));
            write_module(&path, module).with_context(|| package.to_owned())?;
        }

        Ok(())
    }
}

fn write_module(path: &std::path::Path, module: Vec<syn::Item>) -> anyhow::Result<()> {
    let file = std::fs::read_to_string(path).context("read")?;
    let mut file = syn::parse_file(&file).context("parse")?;

    file.items.extend(module);
    std::fs::write(path, prettyplease::unparse(&file)).context("write")?;

    Ok(())
}
