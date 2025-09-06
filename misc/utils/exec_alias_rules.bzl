"""Alias that transitions its target to `compilation_mode=opt`.  Use `transition_alias="opt"` to enable."""

def _transition_alias_impl(ctx):
    # `ctx.attr.actual` is a list of 1 item due to the transition
    output = ctx.actions.declare_file("{}_{}".format(ctx.label.name, ctx.executable.actual.basename))
    ctx.actions.symlink(output = output, target_file = ctx.executable.actual, is_executable = True)

    actual = ctx.attr.actual[0] # type: Target

    providers = [
        DefaultInfo(
            executable = output,
            default_runfiles = actual[DefaultInfo].default_runfiles,
        )
    ] # type: list

    if RunEnvironmentInfo in actual:
        providers.append(actual[RunEnvironmentInfo])

    return providers

def _change_compilation_mode(compilation_mode):
    def _change_compilation_mode_impl(_settings, _attr):
        return {
            "//command_line_option:compilation_mode": compilation_mode,
        }

    return transition(
        implementation = _change_compilation_mode_impl,
        inputs = [],
        outputs = [
            "//command_line_option:compilation_mode",
        ],
    )

def _transition_alias_rule(compilation_mode):
    return rule(
        implementation = _transition_alias_impl,
        attrs = {
            "actual": attr.label(
                mandatory = True,
                executable = True,
                cfg = _change_compilation_mode(compilation_mode),
            ),
            "_allowlist_function_transition": attr.label(
                default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
            ),
        },
        executable = True,
        doc = "Transitions a Rust library crate to the `compilation_mode=opt`.",
    )

exec_transition_alias_dbg = _transition_alias_rule("dbg")
exec_transition_alias_fastbuild = _transition_alias_rule("fastbuild")
exec_transition_alias_opt = _transition_alias_rule("opt")
