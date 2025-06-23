load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "openapiv3_1",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test_deps = ["//crates/flv"],
    deps = [
        "//crates/amf0",
        "//crates/bytes-util",
        "//crates/context",
        "//crates/future-ext",
        "//crates/nutype-enum",
    ],
    test_data = ["//assets:mp4"]
)
