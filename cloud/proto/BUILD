load("//bazel/tools:package.bzl", "scuffle_package", "scuffle_build_script")

exports_files(["Cargo.toml"])

scuffle_build_script(
    name = "build_script",
    data = glob(["pb/**"]),
    deps = ["//crates/tinc/build"],
)

scuffle_package(
    crate_name = "scufflecloud-proto",
    deps = [
        "//crates/tinc",
        ":build_script",
    ],
)
