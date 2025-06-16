load(":ffmpeg/wrapper.bzl", "make_ffmpeg_wrapper")

[
    make_ffmpeg_wrapper(
        name = tool,
        binary = select({
            "//build/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:" + tool + "_tool",
            "//build/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:" + tool + "_tool",
            "//build/platforms:macos-x86_64": "@sysrooffmpegts-macos-universal//:" + tool + "_tool",
            "//build/platforms:macos-aarch64": "@ffmpeg-macos-universal//:" + tool + "_tool",
        }),
        libs = "//build/toolchains/ffmpeg:lib",
        tags = ["manual", "no-cache"],
        visibility = ["//visibility:public"],
    )
    for tool in ["ffmpeg", "ffprobe"]
]
