load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

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
        data = ["@scuffle_assets//:mp4", "//build/tools:ffprobe"],
        insta = True,
        env = {
            "FFPROBE": "$(rootpath //build/tools:ffprobe)",
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:mp4)",
        },
    ),
)
