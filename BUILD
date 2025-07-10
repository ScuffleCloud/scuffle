load("@bazel_skylib//rules:common_settings.bzl", "string_flag")

exports_files(["Cargo.toml", "Cargo.lock", "rustfmt.toml", ".config/nextest.toml"])

string_flag(
    name = "test_profile",
    build_setting_default = "default",
    values = ["ci", "default"],
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

string_flag(
    name = "test_workspace_root",
    build_setting_default = "",
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)
