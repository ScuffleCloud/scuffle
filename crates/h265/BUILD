load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-h265",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test = scuffle_test(
        insta = True,
    ),
    deps = [
        "//crates/bytes-util",
        "//crates/expgolomb",
        "//crates/nutype-enum",
    ],
)
