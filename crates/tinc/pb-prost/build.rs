use prost::Message;
use prost_types::FileDescriptorSet;

fn main() {
    println!("cargo:rerun-if-changed=./annotations.proto");

    let descriptor_out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("tinc.annotations.pb");

    if let Ok(pre_compiled_fd) = std::env::var("TINC_COMPILED_FD") {
        let data = std::fs::read(pre_compiled_fd).unwrap();
        prost_build::Config::new()
            .compile_fds(FileDescriptorSet::decode(data.as_slice()).unwrap())
            .unwrap_or_else(|e| panic!("Failed to compile annotations.proto: {e}"));

        std::fs::write(descriptor_out, data).unwrap();
    } else {
        prost_build::Config::new()
            .file_descriptor_set_path(descriptor_out)
            .compile_protos(&["./annotations.proto"], &["."])
            .unwrap_or_else(|e| panic!("Failed to compile annotations.proto: {e}"));
    }
}
