load("//build/tools:package.bzl", "scuffle_build_script", "scuffle_package")
exports_files(["Cargo.toml"])

scuffle_build_script(
    name = "build_script",
    data = [":annotations.proto", "//crates/tinc:protobuf"],
    env = {
        "TINC_COMPILED_FD": "$(location //crates/tinc:protobuf)",
    },
)

scuffle_package(
    crate_name = "tinc-pb-prost",
    compile_data = [
        ":Cargo.toml",
    ],
    deps = [":build_script"],
)
