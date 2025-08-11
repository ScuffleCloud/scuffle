load("@rules_shell//shell:sh_test.bzl", "sh_test")

load("//build/utils/rust:package.bzl", "scuffle_package")
load("//build/utils/rust:manifest.bzl", "cargo_toml")
load("//build/utils:exec_alias_rules.bzl", "exec_transition_alias_opt")

cargo_toml()

exec_transition_alias_opt(
    name = "cargo-deny",
    actual = "@bindeps//:cargo-deny__cargo-deny",
    tags = ["manual"],
)

scuffle_package(
    crate_name = "deny_runner",
    crate_type = "bin",
    tags = ["manual"],
    visibility = ["//visibility:public"],
    extra_target_kwargs = {
        "data": [":cargo-deny", "//tools:cargo", "//:deny.toml"],
        "env": {
            "CARGO_BIN": "$(rlocationpath //tools:cargo)",
            "CARGO_DENY_BIN": "$(rlocationpath :cargo-deny)",
            "CARGO_DENY_CONFIG": "$(rlocationpath //:deny.toml)",
        },
    }
)
