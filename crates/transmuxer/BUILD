load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

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
        data = ["@scuffle_assets//:flv", "//build/tools:ffprobe"],
        env = {
            "FFPROBE": "$(rootpath //build/tools:ffprobe)",
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:flv)",
        },
    ),
)
