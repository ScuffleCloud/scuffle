load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-batching",
    proc_macro_deps = ["//crates/changelog"],
    data = [
        ":Cargo.toml",
        ":CHANGELOG.md",
    ],
)
