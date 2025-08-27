load("//misc/utils/rust:package.bzl", "scuffle_package", "scuffle_test")
load("//misc/utils/rust:postcompile.bzl", "postcompile_deps")
load("//misc/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

postcompile_deps(
    name = "postcompile_deps",
    deps = [":metrics"],
)

scuffle_package(
    crate_name = "scuffle-metrics",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = [
        "//crates/changelog",
        "//crates/metrics/derive",
    ],
    test = scuffle_test(
        deps = ["//crates/postcompile"],
        data = [":postcompile_deps"],
        insta = True,
    ),
)
