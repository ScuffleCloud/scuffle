load("//build/utils/rust:package.bzl", "scuffle_build_script", "scuffle_package")
load("@protobuf//bazel:proto_library.bzl", "proto_library")
load("//build/utils:proto_descriptor_set.bzl", "proto_descriptor_set")
load("//build/utils/rust:manifest.bzl", "cargo_toml")

cargo_toml()

proto_library(
    name = "pb",
    srcs = glob(["pb/**"]),
    deps = [
        "//crates/tinc:protobuf",
        "@protobuf//:empty_proto",
        "@protobuf//:timestamp_proto",
        "@protobuf//:descriptor_proto",
    ],
    strip_import_prefix = "/cloud/proto/pb",
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
        "TINC_SCUFFLE_CLOUD_COMPILED_FD": "$(location :tinc_fds)",
    },
)

scuffle_package(
    crate_name = "scufflecloud-proto",
    deps = [
        "//crates/tinc",
        ":build_script"
    ],
)
