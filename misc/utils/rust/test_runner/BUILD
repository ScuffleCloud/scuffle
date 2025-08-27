load("//misc/utils/rust:package.bzl", "scuffle_package")
load("//misc/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "test_runner",
    crate_type = "bin",
    srcs = ["src/main.rs"],
    deps = ["//misc/utils/rust/test_runner/lib"],
    tags = ["manual"],
)
