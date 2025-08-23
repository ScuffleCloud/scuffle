load("@bazel_skylib//rules:common_settings.bzl", "string_flag", "bool_flag")
load("//settings:rustc_flags_file.bzl", "rustc_flags_file")

string_flag(
    name = "test_profile",
    build_setting_default = "default",
    values = ["ci", "default"],
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

bool_flag(
    name = "test_insta_force_pass",
    build_setting_default = False,
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

bool_flag(
    name = "test_valgrind",
    build_setting_default = False,
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

rustc_flags_file(
    name = "test_rustc_flags",
    build_setting_default = [],
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)

bool_flag(
    name = "cargo_bazel_vendor",
    build_setting_default = False,
    tags = ["manual", "no-cache"],
    visibility = ["//visibility:public"],
)
