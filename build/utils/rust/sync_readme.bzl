load("@rules_rust//rust:rust_common.bzl", "CrateInfo")
load("//build/utils/rust:rustdoc.bzl", "RustDocInfo")

def _init_SyncReadmeInfo(render_output):
    return {
        "render_output": render_output,
        "_SyncReadmeInfo": "_SyncReadmeInfo",
    }

SyncReadmeInfo, _new_SyncReadmeInfo = provider(
    doc = "A set of clippy json diagnostics.",
    fields = {
        "_SyncReadmeInfo": "A tag to identify this provider by",
        "render_output": "A json file containing the rendered output",
    },
    init = _init_SyncReadmeInfo,
)

def _sync_readme_rule_impl(ctx):
    render_output = ctx.actions.declare_file(ctx.label.name + "_sync_readme.json")

    json_out = ctx.attr.rustdoc[RustDocInfo].json_out
    if not json_out:
        fail("rustdoc must be of type json")

    args = ctx.actions.args()

    args.add("--cargo-toml", ctx.file.cargo_manifest)
    args.add("--rustdoc-json", json_out)
    args.add("--readme-md", ctx.file.readme)
    args.add("--render-output", render_output)

    ctx.actions.run(
        executable = ctx.executable._sync_readme,
        inputs = [
            ctx.file.readme,
            json_out,
            ctx.file.cargo_manifest,
        ],
        outputs = [render_output],
        env = {},
        arguments = [args],
        mnemonic = "SyncReadme",
        progress_message = "SyncReadme %{label}",
    )

    return [
        DefaultInfo(files = depset([render_output])),
        OutputGroupInfo(
            sync_readme = [render_output],
        ),
        SyncReadmeInfo(
            render_output = render_output,
        ),
    ]

sync_readme = rule(
    attrs = {
        "readme": attr.label(
            allow_single_file = True,
        ),
        "cargo_manifest": attr.label(
            allow_single_file = True,
        ),
        "rustdoc": attr.label(
            providers = [RustDocInfo],
        ),
        "_sync_readme": attr.label(
            default = "//build/utils/rust/sync_readme",
            cfg = "exec",
            executable = True,
        ),
    },
    implementation = _sync_readme_rule_impl,
)


def _sync_readme_test_rule_impl(ctx):
    wrapper = ctx.actions.declare_file(ctx.label.name + "_" +  ctx.executable._sync_readme_test.basename)
    ctx.actions.symlink(
        output = wrapper,
        target_file = ctx.executable._sync_readme_test,
    )

    render_output = ctx.attr.sync_readme[SyncReadmeInfo].render_output

    return [
        DefaultInfo(
            executable = wrapper,
            files = depset([wrapper]),
            runfiles = ctx.runfiles(files = [render_output]),
        ),
        RunEnvironmentInfo(
            environment = {
                "RENDER_OUTPUT_PATH": render_output.short_path,
                "CLICOLOR_FORCE": "1",
            },
        ),
    ]

sync_readme_test = rule(
    attrs = {
        "sync_readme": attr.label(
            providers = [SyncReadmeInfo],
        ),
        "_sync_readme_test": attr.label(
            default = Label("//build/utils/rust/sync_readme/test_runner"),
            executable = True,
            cfg = "exec",
        ),
    },
    implementation = _sync_readme_test_rule_impl,
    test = True,
)
