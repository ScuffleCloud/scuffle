def _cargo_metadata_impl(ctx):
    metadata = ctx.actions.declare_file(ctx.label.name + "_cargo_manifest.json")

    env = ctx.attr.env | {
        "CARGO": ctx.executable._cargo.path,
        "CARGO_MANIFEST": ctx.attr.manifest[DefaultInfo].files.to_list()[0].path,
        "METADATA_OUT": metadata.path,
    }

    inputs = []
    for item in ctx.attr.data:
        inputs.extend(item[DefaultInfo].files.to_list())
        inputs.extend(item[DefaultInfo].default_runfiles.files.to_list())

    ctx.actions.run(
        executable = ctx.executable._cargo_metadata_runner,
        mnemonic = "CargoMetadata",
        outputs = [metadata],
        inputs = inputs,
        arguments = ["--"] + ctx.attr.args,
        env = env,
        tools = [ctx.executable._cargo]
    )

    return DefaultInfo(files = depset([metadata]))

cargo_metadata = rule(
    implementation = _cargo_metadata_impl,
    attrs = {
        "manifest": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "env": attr.string_dict(),
        "args": attr.string_list(),
        "data": attr.label_list(
            allow_files = True,
        ),
        "_cargo_metadata_runner": attr.label(
            executable = True,
            default = "//build/tools/cargo_metadata",
            cfg = "exec",
        ),
        "_cargo": attr.label(
            executable = True,
            default = "//tools/cargo",
            cfg = "exec",
        ),
    }
)