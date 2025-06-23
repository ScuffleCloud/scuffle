load("//bazel/tools:package.bzl", "scuffle_build_script", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_build_script(
    name = "build_script",
    data = glob(["pb/**"]),
    deps = ["//crates/tinc/build"],
)

scuffle_package(
    crate_name = "tinc-integration-tests",
    data = [
        ":Cargo.toml",
    ],
    test_deps = [
        ":build_script",
        "//crates/tinc",
    ],
    test_insta = True,
)
