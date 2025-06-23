load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "xtask",
    crate_type = "bin",
    data = glob(["**/*.md"]),
    tags = ["manual"],
)
