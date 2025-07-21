load("//build/utils:package.bzl", "scuffle_package")
load("@rules_rust//rust/private:rustc.bzl", "error_format")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    name = "test_runner",
    crate_name = "clippy_test_runner",
    crate_type = "bin",
    tags = ["manual"],
)

error_format(
    name = "error_format",
    build_setting_default = "json",
    visibility = ["//build/utils:__subpackages__"]
)
