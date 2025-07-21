load("//build/utils:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils:postcompile.bzl", "postcompile_deps")
load("//build/utils:cargo_toml.bzl", "cargo_toml")

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
