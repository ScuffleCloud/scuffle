load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:postcompile.bzl", "postcompile_deps")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

cargo_toml()

postcompile_deps(
    name = "postcompile_deps",
    deps = ["@cargo_vendor//:tokio"],
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
