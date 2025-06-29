load("@aspect_bazel_lib//lib:copy_to_directory.bzl", "copy_to_directory")

copy_to_directory(
    name = "lib",
    include_external_repositories = ["*"],
    srcs = select({
        "//bazel/toolchains:linux_x86_64": ["@ffmpeg-linux-x86_64//:lib"]
    }),
    root_paths = ["lib"],
    visibility = ["//visibility:public"],
)

copy_to_directory(
    name = "include",
    include_external_repositories = ["*"],
    srcs = select({
        "//bazel/toolchains:linux_x86_64": ["@ffmpeg-linux-x86_64//:include"]
    }),
    root_paths = ["include"],
    visibility = ["//visibility:public"],
)
