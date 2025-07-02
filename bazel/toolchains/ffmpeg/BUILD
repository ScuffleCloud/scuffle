alias(
    name = "lib",
    actual = select({
        "//bazel/toolchains:linux_x86_64": "@ffmpeg-linux-x86_64//:lib"
    }),
    visibility = ["//visibility:public"],
)

alias(
    name = "include",
    actual = select({
        "//bazel/toolchains:linux_x86_64": "@ffmpeg-linux-x86_64//:include"
    }),
    visibility = ["//visibility:public"],
)
