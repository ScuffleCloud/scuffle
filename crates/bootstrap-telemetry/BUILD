load("//bazel/tools:package.bzl", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_package(
    crate_name = "scuffle-bootstrap-telemetry",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test_deps = ["//crates/metrics"],
    deps = [
        "//crates/bootstrap",
        "//crates/context",
        "//crates/http",
        "//crates/pprof",
    ],
)
