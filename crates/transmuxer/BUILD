load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-transmuxer",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = [
        "//crates/aac",
        "//crates/av1",
        "//crates/bytes-util",
        "//crates/flv",
        "//crates/h264",
        "//crates/h265",
        "//crates/mp4",
    ],
    test_data = ["//assets:flv"],
)
