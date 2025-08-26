load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    aliases = {
        "//cloud/proto": "pb",
    },
    crate_type = "bin",
    crate_name = "scufflecloud-core",
)
