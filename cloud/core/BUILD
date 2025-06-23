load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scufflecloud-core",
    aliases = {
        "//cloud/proto": "pb"
    },
    deps = [
        "//cloud/proto",
    ],
)
