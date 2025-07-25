load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("@rules_rust//rust:defs.bzl", "rust_test")
load("//build/utils/rust:postcompile.bzl", "PostcompilerDepsInfo")

def to_dict(**kwargs):
    return kwargs

def _nextest_test_impl(ctx):
    parent_providers = ctx.super()

    default_info = None
    crate_info = None
    run_environment_info = None
    for provider in parent_providers:
        if hasattr(provider, "wrapped_crate_type"):
            crate_info = provider
        elif hasattr(provider, "files"):
            default_info = provider
        elif hasattr(provider, "environment"):
            run_environment_info = provider

    if not crate_info:
        fail("could not find CrateInfo")
    if not default_info:
        fail("could not find DefaultInfo")
    if not run_environment_info:
        fail("could not find RunEnvironmentInfo")

    test_binary = default_info.files.to_list()[0]

    env = {
        "RUNNER_CRATE": crate_info.name,
        "RUNNER_BINARY": test_binary.short_path,
        "COVERAGE_BINARY": test_binary.short_path,
        "RUNNER_CONFIG": ctx.attr._nextest_config[DefaultInfo].files.to_list()[0].short_path,
        "RUNNER_PROFILE": ctx.attr._nextest_profile[BuildSettingInfo].value,
    }

    env.update(run_environment_info.environment)

    for data in ctx.attr.data:
        if PostcompilerDepsInfo in data:
            env.update(data[PostcompilerDepsInfo].env)

    # Add workspace root if specified
    if ctx.attr.workspace_root:
        env["INSTA_WORKSPACE_ROOT"] = ctx.attr.workspace_root[BuildSettingInfo].value

    runfiles = ctx.runfiles(files = ctx.files.data + ctx.attr._nextest_config[DefaultInfo].files.to_list() + [test_binary, ctx.executable._process_wrapper])
    runfiles = runfiles.merge(default_info.default_runfiles)
    runfiles = runfiles.merge(ctx.attr._test_runner[DefaultInfo].default_runfiles)
    runfiles = runfiles.merge_all([dep[DefaultInfo].default_runfiles for dep in ctx.attr.data])

    is_windows = ctx.target_platform_has_constraint(ctx.attr._windows_constraint[platform_common.ConstraintValueInfo])
    if is_windows:
        wrapper_script = ctx.actions.declare_file(ctx.label.name + ".bat")
        ctx.actions.write(
            output = wrapper_script,
            content = '@"{}" --subst "pwd=${{pwd}}" -- "{}" %*'.format(
                ctx.executable._process_wrapper.short_path,
                ctx.executable._test_runner.short_path
            ),
            is_executable = True,
        )
    else:
        wrapper_script = ctx.actions.declare_file(ctx.label.name + ".sh")
        ctx.actions.write(
            output = wrapper_script,
            content = '#!/usr/bin/env bash\nexec "{}" --subst \'pwd=${{pwd}}\' -- "{}" $@'.format(
                ctx.executable._process_wrapper.short_path,
                ctx.executable._test_runner.short_path,
            ),
            is_executable = True,
        )

    parent_providers.remove(default_info)
    parent_providers.remove(run_environment_info)

    return [
        DefaultInfo(
            executable = wrapper_script,
            files = default_info.files,
            runfiles = runfiles,
        ),
        RunEnvironmentInfo(
            environment = env,
            inherited_environment = run_environment_info.inherited_environment
        ),
    ] + parent_providers

nextest_test = rule(
    doc = "A test rule that runs tests using a custom test runner.",
    implementation = _nextest_test_impl,
    parent = rust_test,
    attrs = {
        "workspace_root": attr.label(mandatory = False),
        "_nextest_profile": attr.label(mandatory = False, default = "//:test_profile"),
        "_test_runner": attr.label(
            default = "//build/utils/rust/test_runner",
            executable = True,
            cfg = "exec"
        ),
        "_nextest_config": attr.label(
            default = "//:.config/nextest.toml",
            allow_single_file = True
        ),
        "_windows_constraint": attr.label(
            default = "@platforms//os:windows"
        ),
    },
)
