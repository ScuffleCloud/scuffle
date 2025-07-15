load("@bazel_skylib//rules:common_settings.bzl", "string_flag")
load("//:cargo_targets.bzl", "cargo_manifests", "cargo_targets", "cargo_workspace_manifest", "cargo_lock")
load("//build/tools:cargo_metadata.bzl", "cargo_metadata")

exports_files(["Cargo.toml", "Cargo.lock", "rustfmt.toml", ".config/nextest.toml", "deny.toml"])

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

# rustdoc(
#     name = "rustdoc",
#     crates = cargo_targets(exclude = [
#         "//tools/cargo/clippy",
#         "//build/tools/clippy",
#         "//build/protobuf",
#     ]),
#     visibility = ["//visibility:public"],
# )
