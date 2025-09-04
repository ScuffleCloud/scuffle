def _rlocationpath(file, workspace_name):
    if file.short_path.startswith("../"):
        return file.short_path[len("../"):]

    return "{}/{}".format(workspace_name, file.short_path)


def _libtool_darwin_wrapper_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    sh_toolchain = ctx.toolchains["@bazel_tools//tools/sh:toolchain_type"]

    ctx.actions.expand_template(
        output = out,
        template = ctx.file._wrapper,
        is_executable = True,
        substitutions = {
            "%%BINARY%%": _rlocationpath(ctx.executable.tool, ctx.workspace_name),
            "%%WORKSPACE_NAME%%": ctx.workspace_name,
            "#!/usr/bin/env bash": "#!{}".format(sh_toolchain.path),
        },
    )

    runfiles = (
        ctx.runfiles(files = [ctx.executable.tool])
            .merge(ctx.attr.tool[DefaultInfo].default_runfiles)
    )

    return [
        DefaultInfo(files = depset([out]), executable = out, runfiles = runfiles),
    ]

libtool_darwin_wrapper = rule(
    implementation = _libtool_darwin_wrapper_impl,
    attrs = {
        "tool": attr.label(executable = True, allow_single_file = True, cfg = "exec"),
        "_wrapper": attr.label(default = ":libtool-darwin-wrapper.sh", allow_single_file = True, executable = True, cfg = "exec"),
    },
    toolchains = ["@bazel_tools//tools/sh:toolchain_type"],
    executable = True,
)
