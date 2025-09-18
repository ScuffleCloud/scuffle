def _render_preview_rule_impl(ctx):
    output_dir = ctx.actions.declare_directory(ctx.label.name)
    ctx.actions.run(
        outputs = [output_dir],
        inputs = [],
        executable = ctx.executable._render_preview,
        arguments = [output_dir.path],
        mnemonic = "RenderEmailPreview",
        progress_message = "RenderEmailPreview %{label}",
    )

    return [
        DefaultInfo(files = depset([output_dir])),
    ]

render_preview = rule(
    attrs = {
        "_render_preview": attr.label(
            default = ":bin",
            cfg = "exec",
            executable = True,
        ),
    },
    implementation = _render_preview_rule_impl,
)
