load("//build/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    aliases = {
        "//cloud/proto": "pb",
    },
    crate_name = "scufflecloud-core",
    crate_type = "bin",
    deps = [
        "//cloud/proto",
    ],
)
