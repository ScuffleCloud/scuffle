load("//build/utils:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-flv",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test = scuffle_test(
        data = [
            "@scuffle_assets//:flv",
        ],
        env = {
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:flv)",
        },
        insta = True,
    ),
    deps = [
        "//crates/aac",
        "//crates/amf0",
        "//crates/av1",
        "//crates/bytes-util",
        "//crates/h264",
        "//crates/h265",
        "//crates/nutype-enum",
    ],
)
