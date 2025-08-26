def _rustc_flags_file_impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".rustc-args.txt")

    ctx.actions.write(
        out,
        "\n".join(ctx.build_setting_value),
    )

    return [
        DefaultInfo(files = depset([out])),
    ]

rustc_flags_file = rule(
    implementation = _rustc_flags_file_impl,
    build_setting = config.string_list(flag = True),
)
