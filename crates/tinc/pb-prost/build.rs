fn main() {
    println!("cargo:rerun-if-changed=./annotations.proto");
    println!("{}", std::env::current_dir().unwrap().display());
    prost_build::Config::new()
        .file_descriptor_set_path(std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("tinc.annotations.pb"))
        .compile_protos(&["./annotations.proto"], &["."])
        .unwrap_or_else(|e| panic!("Failed to compile annotations.proto: {e}"));
}
