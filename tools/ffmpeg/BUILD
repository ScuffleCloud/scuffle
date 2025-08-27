load("//misc/utils:binary_wrapper.bzl", "binary_wrapper")
load("@bazel_skylib//lib:selects.bzl", "selects")

[
    binary_wrapper(
        name = tool,
        binary = select({
            "//misc/platforms:linux-x86_64": "@ffmpeg-linux-x86_64//:" + tool + "_tool",
            "//misc/platforms:linux-aarch64": "@ffmpeg-linux-aarch64//:" + tool + "_tool",
            "//misc/platforms:windows-x86_64": "@ffmpeg-windows-x86_64//:" + tool + "_tool",
            "//misc/platforms:windows-aarch64": "@ffmpeg-windows-aarch64//:" + tool + "_tool",
        }),
        env = selects.with_or({
            ("//misc/platforms:linux-x86_64", "//misc/platforms:linux-aarch64"): {
                "LD_LIBRARY_PATH": "$$RUNFILES/$(rlocationpath //misc/toolchains/ffmpeg:lib):$$LD_LIBRARY_PATH",
            },
            "//conditions:default": {},
        }),
        data = ["//misc/toolchains/ffmpeg:lib"],
        tags = ["manual", "no-cache"],
        visibility = ["//visibility:public"],
    )
    for tool in ["ffmpeg", "ffprobe"]
]
