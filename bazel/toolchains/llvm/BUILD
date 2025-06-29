load("@aspect_bazel_lib//lib:copy_to_directory.bzl", "copy_to_directory")

copy_to_directory(
    name = "libclang",
    include_external_repositories = ["*"],
    srcs = select({
        "//bazel/toolchains:linux_x86_64": ["@libclang-linux-x86_64//:lib"]
    }),
    root_paths = ["lib"],
    visibility = ["//visibility:public"],
)

copy_to_directory(
    name = "clang",
    include_external_repositories = ["*"],
    srcs = select({
        "//bazel/toolchains:linux_x86_64": ["@libclang-linux-x86_64//:bin"]
    }),
    root_paths = ["bin"],
    visibility = ["//visibility:public"],
)

copy_to_directory(
    name = "sysroot",
    include_external_repositories = ["*"],
    srcs = select({
        "//bazel/toolchains:linux_x86_64": ["@linux-x86_64-sysroots//:sysroot"],
    }),
    visibility = ["//visibility:public"],
)
