load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-ffmpeg",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = ["//crates/nutype-enum"],
    test_deps = ["//crates/mp4"],
    test_insta = True,
    test_data = ["//assets:mp4"],
)
