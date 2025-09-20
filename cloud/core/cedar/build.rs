fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));

    println!("cargo::rerun-if-changed=../static_policies.cedarschema");
    let policy_file = manifest_dir.join("../static_policies.cedarschema");

    let schema_content = std::fs::read_to_string(policy_file).expect("failed to read policy file");

    let output = scuffle_cedar_policy_codegen::Config::new()
        .generate_from_schema(&schema_content)
        .expect("failed to compile cedar schema");

    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("missing OUT_DIR"));

    let output_path = out_dir.join("static_policies.cedar.rs");

    std::fs::write(output_path, output.to_string()).expect("failed to write output");
}
