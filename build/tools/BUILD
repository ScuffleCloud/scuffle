load(":ffmpeg/wrapper.bzl", "make_ffmpeg_wrapper")

[
    make_ffmpeg_wrapper(
        name = tool,
        binary = select({
            "//build/platforms:linux_x86_64": "@ffmpeg-linux-x86_64//:{}_tool".format(tool),
            "//build/platforms:linux_aarch64": "@ffmpeg-linux-aarch64//:{}_tool".format(tool),
        }),
        libs = "//build/toolchains/ffmpeg:lib",
        tags = ["manual", "no-cache"],
        visibility = ["//visibility:public"],
    )
    for tool in ["ffmpeg", "ffprobe", "ffplay"]
]
