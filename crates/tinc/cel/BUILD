load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "tinc-cel",
    data = [
        ":Cargo.toml",
    ],
    test_insta = True,
)
