load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_doctest_builder",
    crate_type = "bin",
    deps = ["//build/utils/rust/doc/test/common"],
    tags = ["manual"],
)
