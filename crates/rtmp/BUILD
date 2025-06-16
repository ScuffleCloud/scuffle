load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-rtmp",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test = scuffle_test(
        data = ["@scuffle_assets//:mp4", "//build/tools:ffmpeg"],
        deps = ["//crates/flv"],
        env = {
            "FFMPEG": "$(rootpath //build/tools:ffmpeg)",
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:mp4)",
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
