load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "tinc",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
        ":annotations.proto",
    ],
    proc_macro_deps = [
        "//crates/changelog",
        "//crates/tinc/derive",
    ],
    test_deps = ["//crates/flv"],
    deps = [
        "//crates/context",
        "//crates/openapiv3_1",
        "//crates/tinc/cel",
    ],
)
