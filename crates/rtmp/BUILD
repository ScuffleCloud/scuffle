load("//bazel/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-rtmp",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test = scuffle_test(
        data = ["//assets:mp4", "//bazel/tools:ffmpeg"],
        deps = ["//crates/flv"],
        env = {
            "FFMPEG": "$(rootpath //bazel/tools:ffmpeg)",
        },
    ),
    deps = [
        "//crates/amf0",
        "//crates/bytes-util",
        "//crates/context",
        "//crates/future-ext",
        "//crates/nutype-enum",
    ],
)
