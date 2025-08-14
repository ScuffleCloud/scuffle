def _make_ffmpeg_wrapper_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    sh_toolchain = ctx.toolchains["@bazel_tools//tools/sh:toolchain_type"]

    ctx.actions.expand_template(
        output = out,
        template = ctx.file._template_file,
        substitutions = {
            "%%BINARY%%": ctx.executable.binary.short_path,
            "#!/usr/bin/env bash": "#!{}".format(sh_toolchain.path),
        },
    )

    runfiles = ctx.runfiles(files = [ctx.executable.binary], transitive_files = ctx.attr.libs.files)

    return [
        DefaultInfo(files = depset([out]), executable = out, runfiles = runfiles),
    ]

make_ffmpeg_wrapper = rule(
    implementation = _make_ffmpeg_wrapper_impl,
    attrs = {
        "binary": attr.label(
            executable = True,
            cfg = "target",
        ),
        "libs": attr.label(),
        "_template_file": attr.label(default = ":ffmpeg/wrapper.sh", allow_single_file = True),
    },
    toolchains = ["@bazel_tools//tools/sh:toolchain_type"],
    executable = True,
)
