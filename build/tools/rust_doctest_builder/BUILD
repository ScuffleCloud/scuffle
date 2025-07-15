load("//build/tools:package.bzl", "scuffle_package")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_doctest_builder",
    crate_type = "bin",
    deps = ["//build/tools/rust_doctest_common"],
    tags = ["manual"],
)
