load("//build/platforms:defs.bzl", "platform_select")

# Currently our llvm builds do not support windows.
os = ["linux", "macos"]

alias(
    name = "libclang",
    actual = select(platform_select("@libclang-{os}-{arch}//:lib", os = os)),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "clang",
    actual = select(platform_select("@libclang-{os}-{arch}//:lib", os = os)),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "sysroot",
    actual = select(platform_select("@{os}-{arch}-sysroots//:sysroot_dir", os = os)),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
