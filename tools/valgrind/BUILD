load("//build/utils:binary_wrapper.bzl", "binary_wrapper")
load("@rules_shell//shell:sh_binary.bzl", "sh_binary")
load("@bazel_skylib//lib:selects.bzl", "selects")

alias(
    name = "libexec",
    actual = select({
        "//build/platforms:linux-x86_64": "@valgrind-linux-x86_64//:libexec",
        "//build/platforms:linux-aarch64": "@valgrind-linux-aarch64//:libexec",
    }),
    tags = ["manual", "no-cache"],
)

binary_wrapper(
    name = "valgrind_bin",
    binary = select({
        "//build/platforms:linux-x86_64": "@valgrind-linux-x86_64//:valgrind",
        "//build/platforms:linux-aarch64": "@valgrind-linux-aarch64//:valgrind",
    }),
    env = {
        "VALGRIND_LIB": "$$RUNFILES/$(rlocationpath :libexec)/valgrind",
    },
    extra_commands = [
        "ulimit -n 4096",
    ],
    data = [":libexec"],
    tags = ["manual", "no-cache"],
)

sh_binary(
    name = "valgrind_dummy",
    srcs = ["dummy.sh"],
)

alias(
    name = "valgrind",
    actual = selects.with_or({
        ("//build/platforms:linux-x86_64", "//build/platforms:linux-aarch64"): ":valgrind_bin",
        "//conditions:default": ":valgrind_dummy"
    }),
    visibility = ["//visibility:public"],
)
