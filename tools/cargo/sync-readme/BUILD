load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    name = "fix",
    crate_name = "sync_readme_fixer",
    crate_type = "bin",
    deps = ["//build/utils/rust/sync_readme/common"],
    tags = ["manual"],
    visibility = ["//visibility:public"],
)
