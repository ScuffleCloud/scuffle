load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-bootstrap",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = [
        "//crates/changelog",
        "//crates/bootstrap/derive",
    ],
    test_deps = [
        "//crates/future-ext",
        "//crates/postcompile",
        "//crates/settings",
        "//crates/signal",
    ],
    deps = ["//crates/context"],
    test_insta = True,
)
