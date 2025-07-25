load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    name = "fix",
    crate_name = "clippy_fixer",
    crate_type = "bin",
    tags = ["manual"],
    visibility = ["//visibility:public"],
)
