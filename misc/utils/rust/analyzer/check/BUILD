load("//misc/utils/rust:package.bzl", "scuffle_package")
load("//misc/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_analyzer_check",
    crate_type = "bin",
    tags = ["manual"],
    visibility = ["//visibility:public"],
)
