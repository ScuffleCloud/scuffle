load("@bazel_skylib//rules:common_settings.bzl", "string_flag")
load("//:cargo_targets.bzl", "cargo_manifests", "cargo_targets", "cargo_workspace_manifest", "cargo_lock")
load("//build/utils/rust:rustdoc.bzl", "rustdoc", "rustdoc_merge")

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

rustdoc(
    name = "runfiles_doc",
    crate = "@rules_rust//rust/runfiles",
)

html_docs = cargo_targets(target = "{name}_doc", exclude = [
    "//tools/cargo/clippy",
    "//tools/cargo/sync-readme",
    "//build/utils/protobuf",
    "//build/utils/rust/clippy",
]) + [
    "//tools/cargo/clippy:fix_doc",
    "//tools/cargo/sync-readme:fix_doc",
    "//build/utils/protobuf:file_concat_doc",
    "//build/utils/rust/clippy:test_runner_doc",
    ":runfiles_doc",
]

json_docs = cargo_targets(target = "{name}_doc_json", exclude = [
    "//tools/cargo/clippy",
    "//tools/cargo/sync-readme",
    "//build/utils/protobuf",
    "//build/utils/rust/clippy",
]) + [
    "//tools/cargo/clippy:fix_doc_json",
    "//tools/cargo/sync-readme:fix_doc_json",
    "//build/utils/rust/clippy:test_runner_doc_json",
    "//build/utils/protobuf:file_concat_doc_json",
]

rustdoc_merge(
    name = "rustdoc",
    targets = html_docs + json_docs,
    visibility = ["//visibility:public"],
)
