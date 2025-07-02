alias(
    name = "lib",
    actual = select({
        "//build/platforms:linux_x86_64": "@ffmpeg-linux-x86_64//:lib"
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "include",
    actual = select({
        "//build/platforms:linux_x86_64": "@ffmpeg-linux-x86_64//:include"
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
