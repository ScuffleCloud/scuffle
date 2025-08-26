load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "test_runner_lib",
    srcs = ["src/lib.rs"],
    tags = ["manual"],
)
