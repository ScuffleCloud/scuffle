load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-settings",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = ["//crates/bootstrap"],
    test = scuffle_test(
        data = glob(["assets/*"]),
        env = {
            "ASSETS_DIR": "crates/settings/assets"
        }
    ),
)
