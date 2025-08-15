load("//build/utils:binary_wrapper.bzl", "binary_wrapper")

alias(
    name = "libexec",
    actual = select({
        "//build/platforms:linux-x86_64": "@valgrind-linux-x86_64//:libexec",
        "//build/platforms:linux-aarch64": "@valgrind-linux-aarch64//:libexec",
    }),
    tags = ["manual", "no-cache"],
)

binary_wrapper(
    name = "valgrind",
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
    visibility = ["//visibility:public"],
)
