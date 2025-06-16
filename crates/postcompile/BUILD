load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:postcompile.bzl", "postcompile_deps")

exports_files(["Cargo.toml"])

postcompile_deps(
    name = "postcompile_deps",
    deps = ["//vendor/cargo:tokio"],
)

scuffle_package(
    crate_name = "postcompile",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test = scuffle_test(
        data = [":postcompile_deps"],
        insta = True,
    ),
)
