alias(
    name = "libclang",
    actual = select({
        "//build/platforms:linux_x86_64": "@libclang-linux-x86_64//:lib"
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "clang",
    actual = select({
        "//build/platforms:linux_x86_64": "@libclang-linux-x86_64//:bin"
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "sysroot",
    actual = select({
        "//build/platforms:linux_x86_64": "@linux-x86_64-sysroots//:sysroot_dir",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
