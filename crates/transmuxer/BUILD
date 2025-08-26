load("//build/utils/rust:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-transmuxer",
    compile_data = [
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
    test = scuffle_test(
        data = ["@scuffle_assets//:flv", "//tools/ffmpeg:ffprobe"],
        env = {
            "FFPROBE": "$(rootpath //tools/ffmpeg:ffprobe)",
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:flv)",
        },
    ),
)
