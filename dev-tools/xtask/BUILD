load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "xtask",
    crate_type = "bin",
    compile_data = glob(["**/*.md"]),
    tags = ["manual"],
)
