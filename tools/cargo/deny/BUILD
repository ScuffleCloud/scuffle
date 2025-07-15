load("@rules_shell//shell:sh_test.bzl", "sh_test")

load("//build/tools:package.bzl", "scuffle_package")
load("//build/tools:cargo_toml.bzl", "cargo_toml")

cargo_toml()

scuffle_package(
    crate_name = "deny_runner",
    crate_type = "bin",
    tags = ["manual"],
    visibility = ["//visibility:public"],
    extra_target_kwargs = {
        "data": ["@bindeps//:cargo-deny__cargo-deny", "//tools:cargo", "//:deny.toml"],
        "env": {
            "CARGO_BIN": "$(rlocationpath //tools:cargo)",
            "CARGO_DENY_BIN": "$(rlocationpath @bindeps//:cargo-deny__cargo-deny)",
            "CARGO_DENY_CONFIG": "$(rlocationpath //:deny.toml)",
        },
    }
)
