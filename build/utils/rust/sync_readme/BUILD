load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "sync_readme",
    crate_type = "bin",
    deps = ["//build/utils/rust/sync_readme/common"],
    tags = ["manual"],
    visibility = ["//visibility:public"],
)
