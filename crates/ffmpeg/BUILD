load("//build/utils:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-ffmpeg",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = ["//crates/nutype-enum"],
    test = scuffle_test(
        deps = ["//crates/mp4"],
        insta = True,
        data = ["@scuffle_assets//:mp4", "//build/toolchains/ffmpeg:lib"],
        env = select({
            "@platforms//os:windows": {
                "PATH": "$${pwd}/$(rootpath //build/toolchains/ffmpeg:lib)"
            },
            "//conditions:default": {
                "LD_LIBRARY_PATH": "$${pwd}/$(rootpath //build/toolchains/ffmpeg:lib)"
            },
        }) | {
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:mp4)",
        }
    ),
)
