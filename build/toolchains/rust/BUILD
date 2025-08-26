load("@rules_rust//crate_universe/private:urls.bzl", "CARGO_BAZEL_LABEL")

config_setting(
    name = "cargo_bazel_vendor",
    flag_values = {
        "//settings:cargo_bazel_vendor": "true",
    },
)

alias(
    name = "cargo-bazel",
    actual = select({
        ":cargo_bazel_vendor": CARGO_BAZEL_LABEL,
        "//conditions:default": ":cargo-bazel-precompiled",
    }),
    visibility = ["//visibility:public"],
)

alias(
    name = "cargo-bazel-precompiled",
    actual = select({
        "//build/platforms:linux-aarch64": "@cargo-bazel-linux-aarch64//:cargo-bazel",
        "//build/platforms:linux-x86_64": "@cargo-bazel-linux-x86_64//:bin",
        "//build/platforms:macos-aarch64": "@cargo-bazel-darwin-aarch64//:cargo-bazel",
        "//build/platforms:macos-x86_64": "@cargo-bazel-darwin-x86_64//:cargo-bazel",
        "//build/platforms:windows-aarch64": "@cargo-bazel-windows-aarch64//:cargo-bazel",
        "//build/platforms:windows-x86_64": "@cargo-bazel-windows-x86_64//:cargo-bazel",
        "//conditions:default": CARGO_BAZEL_LABEL,
    }),
)
