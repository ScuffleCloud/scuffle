load("//build/tools:package.bzl", "scuffle_package")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "xtask",
    crate_type = "bin",
    compile_data = glob(["**/*.md"]),
    tags = ["manual"],
)
