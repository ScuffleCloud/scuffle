load("//build/utils/rust:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-mp4",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = [
        "//crates/aac",
        "//crates/av1",
        "//crates/bytes-util",
        "//crates/h264",
        "//crates/h265",
    ],
    test = scuffle_test(
        data = ["@scuffle_assets//:mp4", "//tools/ffmpeg:ffprobe"],
        insta = True,
        env = {
            "FFPROBE": "$(rootpath //tools/ffmpeg:ffprobe)",
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:mp4)",
        },
    ),
)
