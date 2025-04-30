#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use anyhow::Context;
pub mod codegen;

#[cfg(feature = "prost")]
mod prost_explore;

mod types;

#[derive(Debug)]
pub enum Mode {
    #[cfg(feature = "prost")]
    Prost,
}

#[derive(Default, Debug)]
struct PathConfigs {
    btree_maps: Vec<String>,
    bytes: Vec<String>,
    boxed: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    disable_tinc_include: bool,
    mode: Mode,
    paths: PathConfigs,
}

impl Config {
    #[cfg(feature = "prost")]
    pub fn prost() -> Self {
        Self::new(Mode::Prost)
    }

    pub fn new(mode: Mode) -> Self {
        Self {
            disable_tinc_include: false,
            mode,
            paths: PathConfigs::default(),
        }
    }

    pub fn disable_tinc_include(&mut self) -> &mut Self {
        self.disable_tinc_include = true;
        self
    }

    pub fn btree_map(&mut self, path: impl std::fmt::Display) -> &mut Self {
        self.paths.btree_maps.push(path.to_string());
        self
    }

    pub fn bytes(&mut self, path: impl std::fmt::Display) -> &mut Self {
        self.paths.bytes.push(path.to_string());
        self
    }

    pub fn boxed(&mut self, path: impl std::fmt::Display) -> &mut Self {
        self.paths.boxed.push(path.to_string());
        self
    }

    pub fn compile_protos(&mut self, protos: &[&str], includes: &[&str]) -> anyhow::Result<()> {
        match self.mode {
            #[cfg(feature = "prost")]
            Mode::Prost => self.compile_protos_prost(protos, includes),
        }
    }

    #[cfg(feature = "prost")]
    fn compile_protos_prost(&mut self, protos: &[&str], includes: &[&str]) -> anyhow::Result<()> {
        use codegen::prost_sanatize::to_snake;
        use prost_reflect::DescriptorPool;
        use quote::ToTokens;
        use syn::parse_quote;
        use types::ProtoTypeRegistry;

        let out_dir_str = std::env::var("OUT_DIR").context("OUT_DIR must be set, typically set by a cargo build script")?;
        let out_dir = std::path::PathBuf::from(&out_dir_str);
        let ft_path = out_dir.join("tinc.fd.bin");

        let mut config = prost_build::Config::new();
        config.file_descriptor_set_path(&ft_path);

        config.btree_map(self.paths.btree_maps.iter());
        self.paths.boxed.iter().for_each(|path| {
            config.boxed(path);
        });
        config.bytes(self.paths.bytes.iter());

        let mut includes = includes.to_vec();

        if !self.disable_tinc_include {
            let extra_includes = out_dir.join("tinc");
            config.extern_path(".tinc", "::tinc::reexports::tinc_pb");
            std::fs::create_dir_all(&extra_includes).context("failed to create tinc directory")?;
            std::fs::write(extra_includes.join("annotations.proto"), tinc_pb::TINC_ANNOTATIONS)
                .context("failed to write tinc_annotations.rs")?;
            includes.push(&out_dir_str);
        }

        let fds = config.load_fds(protos, &includes).context("failed to generate tonic fds")?;

        let fds_bytes = std::fs::read(ft_path).context("failed to read tonic fds")?;

        let pool = DescriptorPool::decode(&mut fds_bytes.as_slice()).context("failed to decode tonic fds")?;

        let mut extensions = prost_explore::Extensions::new(&pool);

        let mut registry = ProtoTypeRegistry::new();

        extensions
            .process(&pool, &mut registry)
            .context("failed to process extensions")?;

        let mut packages = codegen::generate_modules(&registry)?;

        packages.values_mut().for_each(|package| {
            package.enum_configs().for_each(|(path, enum_config)| {
                enum_config.attributes().for_each(|attribute| {
                    config.enum_attribute(path, attribute.to_token_stream().to_string());
                });
                enum_config.variants().for_each(|variant| {
                    let path = format!("{path}.{variant}");
                    enum_config.variant_attributes(variant).for_each(|attribute| {
                        config.field_attribute(&path, attribute.to_token_stream().to_string());
                    });
                });
            });

            package.message_configs().for_each(|(path, message_config)| {
                message_config.attributes().for_each(|attribute| {
                    config.message_attribute(path, attribute.to_token_stream().to_string());
                });
                message_config.fields().for_each(|field| {
                    let path = format!("{path}.{field}");
                    message_config.field_attributes(field).for_each(|attribute| {
                        config.field_attribute(&path, attribute.to_token_stream().to_string());
                    });
                });
                message_config.oneof_configs().for_each(|(field, oneof_config)| {
                    let path = format!("{path}.{field}");
                    oneof_config.attributes().for_each(|attribute| {
                        // In prost oneofs (container) are treated as enums
                        config.enum_attribute(&path, attribute.to_token_stream().to_string());
                    });
                    oneof_config.fields().for_each(|field| {
                        let path = format!("{path}.{field}");
                        oneof_config.field_attributes(field).for_each(|attribute| {
                            config.field_attribute(&path, attribute.to_token_stream().to_string());
                        });
                    });
                });
            });

            package.extra_items.extend(package.services.iter().flat_map(|service| {
                let mut builder = tonic_build::CodeGenBuilder::new();

                builder.emit_package(true).build_transport(true);

                let service = service
                    .methods
                    .iter()
                    .fold(
                        tonic_build::manual::Service::builder()
                            .name(service.name())
                            .package(&service.package),
                        |service_builder, (name, method)| {
                            let mut builder = tonic_build::manual::Method::builder()
                                .input_type("")
                                .output_type("")
                                .codec_path("")
                                .name(to_snake(name))
                                .route_name(name);

                            if method.input.is_stream() {
                                builder = builder.client_streaming()
                            }

                            if method.output.is_stream() {
                                builder = builder.server_streaming();
                            }

                            service_builder.method(builder.build())
                        },
                    )
                    .build();

                let client = builder.generate_client(&service, "");
                let server = builder.generate_server(&service, "");

                [parse_quote!(#client), parse_quote!(#server)]
            }));
        });

        config.compile_fds(fds).context("prost compile")?;

        for (package, module) in packages {
            let path = out_dir.join(format!("{package}.rs"));
            write_module(&path, module.extra_items).with_context(|| package.to_owned())?;
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
