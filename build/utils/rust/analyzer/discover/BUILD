load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_analyzer_discover",
    crate_type = "bin",
    tags = ["manual"],
    extra_target_kwargs = {
        "data": ["@rules_rust//rust/private:rust_analyzer_detect_sysroot"],
        "env": {
            "RUST_ANALYZER_TOOLCHAIN_PATH": "$(rlocationpath @rules_rust//rust/private:rust_analyzer_detect_sysroot)"
        },
    },
    visibility = ["//visibility:public"],
)
