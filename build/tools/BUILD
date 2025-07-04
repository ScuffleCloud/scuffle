load("//build/platforms:defs.bzl", "platform_select")
load(":ffmpeg/wrapper.bzl", "make_ffmpeg_wrapper")

[
    make_ffmpeg_wrapper(
        name = tool,
        binary = select(platform_select("@ffmpeg-{os}-{arch}//:" + tool + "_tool")),
        libs = "//build/toolchains/ffmpeg:lib",
        tags = ["manual", "no-cache"],
        visibility = ["//visibility:public"],
    )
    for tool in ["ffmpeg", "ffprobe", "ffplay"]
]
