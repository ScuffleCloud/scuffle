load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-context",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test = scuffle_test(
        deps = ["//crates/future-ext"],
    ),
)
