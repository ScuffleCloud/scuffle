load("//bazel/tools:package.bzl", "scuffle_build_script", "scuffle_package")

exports_files(["Cargo.toml"])

scuffle_build_script(
    name = "build_script",
    data = [":annotations.proto"],
)

scuffle_package(
    crate_name = "tinc-pb-prost",
    deps = [":build_script"],
    data = [
        ":Cargo.toml",
    ],
)
