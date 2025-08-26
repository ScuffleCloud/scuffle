alias(
    name = "lib",
    actual = select({
        "//build/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:lib",
        "//build/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:lib",
        "//build/platforms:windows-x86_64": "@ffmpeg-windows-x86_64//:lib",
        "//build/platforms:windows-aarch64": "@ffmpeg-windows-aarch64//:lib",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "include",
    actual = select({
        "//build/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:include",
        "//build/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:include",
        "//build/platforms:windows-x86_64": "@ffmpeg-windows-x86_64//:include",
        "//build/platforms:windows-aarch64": "@ffmpeg-windows-aarch64//:include",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
