load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:postcompile.bzl", "postcompile_deps")

exports_files(["Cargo.toml"])

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
