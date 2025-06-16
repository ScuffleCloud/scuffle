alias(
    name = "lib",
    actual = select({
        "//build/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:lib",
        "//build/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:lib",
        "//build/platforms:macos-x86_64": "@sysrooffmpegts-macos-universal//:lib",
        "//build/platforms:macos-aarch64": "@ffmpeg-macos-universal//:lib",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)

alias(
    name = "include",
    actual = select({
        "//build/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:include",
        "//build/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:include",
        "//build/platforms:macos-x86_64": "@sysrooffmpegts-macos-universal//:include",
        "//build/platforms:macos-aarch64": "@ffmpeg-macos-universal//:include",
    }),
    tags = ["no-cache", "manual"],
    visibility = ["//visibility:public"],
)
