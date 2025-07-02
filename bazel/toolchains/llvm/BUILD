alias(
    name = "libclang",
    actual = select({
        "//bazel/toolchains:linux_x86_64": "@libclang-linux-x86_64//:lib"
    }),
    visibility = ["//visibility:public"],
)

alias(
    name = "clang",
    actual = select({
        "//bazel/toolchains:linux_x86_64": "@libclang-linux-x86_64//:bin"
    }),
    visibility = ["//visibility:public"],
)

alias(
    name = "sysroot",
    actual = select({
        "//bazel/toolchains:linux_x86_64": "@linux-x86_64-sysroots//:sysroot_dir",
    }),
    visibility = ["//visibility:public"],
)
