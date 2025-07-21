load("//build/utils:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

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
        data = ["@scuffle_assets//:flv", "//build/utils:ffprobe"],
        env = {
            "FFPROBE": "$(rootpath //build/utils:ffprobe)",
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:flv)",
        },
    ),
)
