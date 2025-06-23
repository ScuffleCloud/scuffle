load("//bazel/tools:package.bzl", "scuffle_package")
load("//bazel/tools:postcompile.bzl", "postcompile_deps")

exports_files(["Cargo.toml"])

postcompile_deps(
    name = "postcompile_deps",
    deps = ["//vendor:tokio"],
)

scuffle_package(
    crate_name = "postcompile",
    data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
    ],
    proc_macro_deps = ["//crates/changelog"],
    test_insta = True,
    test_data = [":postcompile_deps", "@rules_rust//tools/upstream_wrapper:rustc"],
    test_env = {
        "POSTCOMPILE_DEPS_MANIFEST": "$(location :postcompile_deps)",
        "RUSTC": "$(location @rules_rust//tools/upstream_wrapper:rustc)",
    },
)
