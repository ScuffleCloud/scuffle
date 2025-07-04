load("//build/platforms:defs.bzl", "platform_select")

alias(
    name = "lib",
    actual = select(platform_select("@ffmpeg-{os}-{arch}//:lib")),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "include",
    actual = select(platform_select("@ffmpeg-{os}-{arch}//:include")),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
