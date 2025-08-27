load("//misc/utils/rust:package.bzl", "scuffle_package")
load("//misc/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "sync_readme_test_runner",
    crate_type = "bin",
    deps = ["//misc/utils/rust/sync_readme/common"],
    tags = ["manual"],
    visibility = ["//visibility:public"],
)
