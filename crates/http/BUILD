load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-http",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = ["//crates/context"],
    test = scuffle_test(
        data = [
            "@scuffle_assets//:pem",
        ],
        env = {
            "ASSETS_DIR": "$(rootpath @scuffle_assets//:pem)",
        },
        deps = ["//crates/future-ext"],
    ),
)
