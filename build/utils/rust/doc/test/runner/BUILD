load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_doctest_runner",
    crate_type = "bin",
    deps = ["//build/utils/rust/doc/test/common", "//build/utils/rust/test_runner/lib"],
    tags = ["manual"],
)
