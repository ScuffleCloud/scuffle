load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-bootstrap",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = [
        "//crates/changelog",
        "//crates/bootstrap/derive",
    ],
    deps = ["//crates/context"],
    test = scuffle_test(
        deps = [
            "//crates/future-ext",
            "//crates/postcompile",
            "//crates/settings",
            "//crates/signal",
        ],
        insta = True,
    ),
)
