fn main() {
    println!("cargo:rerun-if-changed=annotations.proto");
    prost_build::compile_protos(&["src/tinc/annotations.proto"], &["."])
        .unwrap_or_else(|e| panic!("Failed to compile annotations.proto: {}", e));
}
