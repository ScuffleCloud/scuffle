
load(":ffmpeg/wrapper.bzl", "make_ffmpeg_wrapper")

[
    make_ffmpeg_wrapper(
        name = tool,
        binary = select({
            "//bazel/toolchains:linux_x86_64": "@ffmpeg-linux-x86_64//:{}_tool".format(tool)
        }),
        libs = "//bazel/toolchains/ffmpeg:lib",
        visibility = ["//visibility:public"],
    )
    for tool in ["ffmpeg", "ffprobe", "ffplay"]
]
