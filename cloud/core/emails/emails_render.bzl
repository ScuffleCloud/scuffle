def _emails_render_rule_impl(ctx):
    output_dir = ctx.actions.declare_directory(ctx.label.name)
    ctx.actions.run(
        outputs = [output_dir],
        inputs = [],
        executable = ctx.executable._emails_render,
        arguments = [output_dir.path],
        mnemonic = "ScuffleCloudCoreEmailsRender",
        progress_message = "ScuffleCloudCoreEmailsRender %{label}",
    )

    return [
        DefaultInfo(files = depset([output_dir])),
    ]

emails_render = rule(
    attrs = {
        "_emails_render": attr.label(
            default = ":bin",
            cfg = "exec",
            executable = True,
        ),
    },
    implementation = _emails_render_rule_impl,
)
