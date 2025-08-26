load("//build/utils/rust:package.bzl", "scuffle_package", "scuffle_test")
load("//build/utils/rust:postcompile.bzl", "postcompile_deps")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

postcompile_deps(
    name = "postcompile_deps",
    deps = ["//crates/tinc"],
)

scuffle_package(
    crate_name = "tinc-build",
    compile_data = [
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/tinc/derive"],
    deps = [
        "//crates/openapiv3_1",
        "//crates/tinc/cel",
        "//crates/tinc/pb-prost",
    ],
    test = scuffle_test(
        deps = ["//crates/postcompile"],
        data = [":postcompile_deps"],
        insta = True,
    ),
)
