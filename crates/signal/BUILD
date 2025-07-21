load("//build/utils:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "scuffle-signal",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    deps = [
        "//crates/bootstrap",
        "//crates/context",
    ],
    test = scuffle_test(
        deps = ["//crates/future-ext"],
    ),
)
