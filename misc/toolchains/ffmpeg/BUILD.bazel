alias(
    name = "lib",
    actual = select({
        "//misc/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:lib",
        "//misc/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:lib",
        "//misc/platforms:windows-x86_64": "@ffmpeg-windows-x86_64//:lib",
        "//misc/platforms:windows-aarch64": "@ffmpeg-windows-aarch64//:lib",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "include",
    actual = select({
        "//misc/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:include",
        "//misc/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:include",
        "//misc/platforms:windows-x86_64": "@ffmpeg-windows-x86_64//:include",
        "//misc/platforms:windows-aarch64": "@ffmpeg-windows-aarch64//:include",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
