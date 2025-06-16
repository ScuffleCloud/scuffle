load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("//build/tools:postcompile.bzl", "postcompile_deps")

exports_files(["Cargo.toml"])

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
