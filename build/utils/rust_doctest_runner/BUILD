load("//build/utils:package.bzl", "scuffle_package")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "rust_doctest_runner",
    crate_type = "bin",
    deps = ["//build/utils/rust_doctest_common", "//build/utils/nextest_runner"],
    tags = ["manual"],
)
