def _make_ffmpeg_wrapper_impl(ctx):
    is_windows = ctx.target_platform_has_constraint(ctx.attr._windows_constraint[platform_common.ConstraintValueInfo])
    ext = ".bat" if is_windows else ".sh"
    template = ctx.attr._template_file_win if is_windows else ctx.attr._template_file

    out = ctx.actions.declare_file(ctx.label.name + ext)
    ctx.actions.expand_template(
        output = out,
        template = template.files.to_list()[0],
        substitutions = {
            "%%BINARY%%": ctx.executable.binary.short_path,
        },
    )

    runfiles = ctx.runfiles(files = [ctx.executable.binary], transitive_files = ctx.attr.libs.files)

    return [
        DefaultInfo(files = depset([out]), executable = out, runfiles = runfiles)
    ]

make_ffmpeg_wrapper = rule(
    implementation = _make_ffmpeg_wrapper_impl,
    attrs = {
        "binary": attr.label(
            executable = True,
            cfg = "target",
        ),
        "libs": attr.label(),
        "_template_file_win": attr.label(default = ":ffmpeg/wrapper.bat", allow_single_file = True),
        "_template_file": attr.label(default = ":ffmpeg/wrapper.sh", allow_single_file = True),
        "_windows_constraint": attr.label(
            default = "@platforms//os:windows"
        ),
    },
    executable = True,
)
