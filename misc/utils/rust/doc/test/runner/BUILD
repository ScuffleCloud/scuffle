load("//misc/utils/rust:package.bzl", "scuffle_package")
load("//misc/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_doctest_runner",
    crate_type = "bin",
    deps = ["//misc/utils/rust/doc/test/common", "//misc/utils/rust/test_runner/lib"],
    tags = ["manual"],
)
