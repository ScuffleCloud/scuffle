load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-flv",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = [
        "//crates/aac",
        "//crates/amf0",
        "//crates/av1",
        "//crates/bytes-util",
        "//crates/h264",
        "//crates/h265",
        "//crates/nutype-enum",
    ],
    test_data = [
        "//assets:flv"
    ],
    test_insta = True,
)
