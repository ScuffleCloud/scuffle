load("//build/tools:package.bzl", "scuffle_package", "scuffle_test")
load("@protobuf//bazel:proto_library.bzl", "proto_library")

exports_files(["Cargo.toml"])

proto_library(
    name = "protobuf",
    srcs = ["annotations.proto"],
    strip_import_prefix = "/crates",
    deps = ["@protobuf//:descriptor_proto"],
    visibility = ["//visibility:public"],
)

scuffle_package(
    crate_name = "tinc",
    compile_data = [
        ":CHANGELOG.md",
        ":Cargo.toml",
        ":annotations.proto",
    ],
    proc_macro_deps = [
        "//crates/changelog",
        "//crates/tinc/derive",
    ],
    deps = [
        "//crates/context",
        "//crates/openapiv3_1",
        "//crates/tinc/cel",
    ],
    test = scuffle_test(
        deps = ["//crates/flv"]
    )
)
