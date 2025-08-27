"""
Wrapper around `rust_test` running with nextest.
"""

load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("@rules_rust//rust:defs.bzl", "rust_test")
load("@rules_rust//rust/private:providers.bzl", _CrateInfo = "CrateInfo")  # @unused
load("//misc/utils/rust:postcompile.bzl", "PostcompilerDepsInfo")

def _nextest_test_impl(ctx):
    parent_providers = ctx.super()

    default_info = None  # type: DefaultInfo | None
    crate_info = None  # type: _CrateInfo | None
    run_environment_info = None  # type: RunEnvironmentInfo | None
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
        "RUNNER_INSTA": "true" if ctx.attr.insta else "false",
        "RUNNER_SOURCE_DIR": ctx.label.package,
    }

    env.update(run_environment_info.environment)

    for data in ctx.attr.data:
        if PostcompilerDepsInfo in data:
            env.update(data[PostcompilerDepsInfo].env)

    # Add workspace root if specified
    if ctx.attr.insta:
        env["INSTA_WORKSPACE_ROOT"] = ""
        if ctx.attr._insta_force_pass[BuildSettingInfo].value:
            env["INSTA_FORCE_PASS"] = "1"

    files = [test_binary, ctx.executable._process_wrapper]
    data = [dep[DefaultInfo].default_runfiles for dep in ctx.attr.data]
    if ctx.attr._valgrind_enabled[BuildSettingInfo].value:
        env["VALGRIND"] = ctx.executable._valgrind.short_path
        files.append(ctx.executable._valgrind)
        data.append(ctx.attr._valgrind[DefaultInfo].default_runfiles)

    runfiles = ctx.runfiles(files = ctx.files.data + ctx.attr._nextest_config[DefaultInfo].files.to_list() + files)
    runfiles = runfiles.merge(default_info.default_runfiles)
    runfiles = runfiles.merge(ctx.attr._test_runner[DefaultInfo].default_runfiles)
    runfiles = runfiles.merge_all(data)

    out = ctx.actions.declare_file(ctx.label.name + ".sh")
    sh_toolchain = ctx.toolchains["@bazel_tools//tools/sh:toolchain_type"]
    ctx.actions.expand_template(
        output = out,
        template = ctx.file._template_file,
        substitutions = {
            "%%PROCESS_WRAPPER%%": ctx.executable._process_wrapper.short_path,
            "%%TARGET_BINARY%%": ctx.executable._test_runner.short_path,
            "#!/usr/bin/env bash": "#!{}".format(sh_toolchain.path),
        },
    )

    parent_providers.remove(default_info)
    parent_providers.remove(run_environment_info)

    return [
        DefaultInfo(
            executable = out,
            files = depset([out], transitive = [default_info.files]),
            runfiles = runfiles,
        ),
        RunEnvironmentInfo(
            environment = env,
            inherited_environment = run_environment_info.inherited_environment,
        ),
    ] + parent_providers

nextest_test = rule(
    doc = "A test rule that runs tests using a custom test runner.",
    implementation = _nextest_test_impl,
    parent = rust_test,
    attrs = {
        "insta": attr.bool(default = False),
        "_nextest_profile": attr.label(mandatory = False, default = "//settings:test_profile"),
        "_insta_force_pass": attr.label(default = "//settings:test_insta_force_pass"),
        "_valgrind_enabled": attr.label(default = "//settings:test_valgrind"),
        "_valgrind": attr.label(default = "//tools/valgrind", cfg = "exec", executable = True),
        "_test_runner": attr.label(
            default = "//misc/utils/rust/test_runner",
            executable = True,
            cfg = "exec",
        ),
        "_nextest_config": attr.label(
            default = "//:.config/nextest.toml",
            allow_single_file = True,
        ),
        "_template_file": attr.label(default = "process_wrapper_tmpl.sh", allow_single_file = True),
    },
    toolchains = ["@bazel_tools//tools/sh:toolchain_type"],
)
