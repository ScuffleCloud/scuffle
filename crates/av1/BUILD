load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-av1",
    proc_macro_deps = ["//crates/changelog"],
    deps = ["//crates/bytes-util"],
    data = [
        ":Cargo.toml",
        ":CHANGELOG.md",
    ],
)
