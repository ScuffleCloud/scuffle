load("//build/utils:binary_wrapper.bzl", "binary_wrapper")

[
    binary_wrapper(
        name = tool,
        binary = select({
            "//build/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:" + tool + "_tool",
            "//build/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:" + tool + "_tool",
            # "//build/platforms:macos-x86_64": "@ffmpegts-macos-universal//:" + tool + "_tool",
            # "//build/platforms:macos-aarch64": "@ffmpeg-macos-universal//:" + tool + "_tool",
        }),
        env = {
            "LD_LIBRARY_PATH": "$$RUNFILES/$(rlocationpath //build/toolchains/ffmpeg:lib):$$LD_LIBRARY_PATH",
        },
        data = ["//build/toolchains/ffmpeg:lib"],
        tags = ["manual", "no-cache"],
        visibility = ["//visibility:public"],
    )
    for tool in ["ffmpeg", "ffprobe"]
]
