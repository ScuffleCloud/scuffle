load("//build/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "xtask",
    crate_type = "bin",
    compile_data = glob(["**/*.md"]),
    tags = ["manual"],
)
