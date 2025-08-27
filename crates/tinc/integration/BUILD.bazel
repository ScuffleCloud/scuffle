load("//misc/utils/rust:package.bzl", "scuffle_build_script", "scuffle_package", "scuffle_test")
load("@protobuf//bazel:proto_library.bzl", "proto_library")
load("//misc/utils:proto_descriptor_set.bzl", "proto_descriptor_set")
load("//misc/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

proto_library(
    name = "pb",
    srcs = glob(["pb/**"]),
    deps = ["//crates/tinc:protobuf", "@protobuf//:duration_proto", "@protobuf//:empty_proto", "@protobuf//:struct_proto", "@protobuf//:timestamp_proto"],
)

proto_descriptor_set(
    name = "tinc_fds",
    deps = [":pb"],
)

scuffle_build_script(
    name = "build_script",
    deps = ["//crates/tinc/build"],
    data = [":tinc_fds"],
    env = {
        "TINC_INTEGRATION_COMPILED_FD": "$(location :tinc_fds)",
    },
)

scuffle_package(
    crate_name = "tinc-integration-tests",
    compile_data = [
        ":Cargo.toml",
    ],
    test = scuffle_test(
        deps = [
            "//crates/tinc",
            ":build_script",
        ],
        insta = True,
    ),
)
