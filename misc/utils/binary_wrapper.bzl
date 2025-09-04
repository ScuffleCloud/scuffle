"""
Helper rule to wrap a binary inside a script file.
"""

load("@rules_rust//rust/private:utils.bzl", "expand_dict_value_locations")

def _rlocationpath(file, workspace_name):
    if file.short_path.startswith("../"):
        return file.short_path[len("../"):]

    return "{}/{}".format(workspace_name, file.short_path)

def _binary_wrapper_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    sh_toolchain = ctx.toolchains["@bazel_tools//tools/sh:toolchain_type"]

    env = expand_dict_value_locations(
        ctx,
        ctx.attr.env,
        ctx.attr.data,
        {},
    )

    export_lines = "\n".join(
        ['export {}="{}"'.format(k, v.replace('"', '\\"')) for k, v in env.items()],
    )

    ctx.actions.expand_template(
        output = out,
        template = ctx.file._template_file,
        is_executable = True,
        substitutions = {
            "%%BINARY%%": _rlocationpath(ctx.executable.binary, ctx.workspace_name),
            "%%WORKSPACE_NAME%%": ctx.workspace_name,
            "%%EXPORT_ENVS%%": export_lines,
            "%%EXTRA_COMMANDS%%": "\n".join(ctx.attr.extra_commands),
            "#!/usr/bin/env bash": "#!{}".format(sh_toolchain.path),
        },
    )

    runfiles = (
        ctx.runfiles(files = [ctx.executable.binary])
            .merge(ctx.attr.binary[DefaultInfo].default_runfiles)
            .merge_all([data[DefaultInfo].default_runfiles.merge(ctx.runfiles(files = data[DefaultInfo].files.to_list())) for data in ctx.attr.data])
    )

    return [
        DefaultInfo(files = depset([out]), executable = out, runfiles = runfiles),
    ]

binary_wrapper = rule(
    implementation = _binary_wrapper_impl,
    attrs = {
        "binary": attr.label(
            executable = True,
            allow_single_file = True,
            cfg = "exec",
        ),
        "extra_commands": attr.string_list(),
        "env": attr.string_dict(),
        "data": attr.label_list(),
        "_template_file": attr.label(default = "//misc/utils:binary_wrapper/template.sh", allow_single_file = True),
    },
    toolchains = ["@bazel_tools//tools/sh:toolchain_type"],
    executable = True,
)
