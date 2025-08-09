load("@bazel_skylib//lib:paths.bzl", "paths")
load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_rust//cargo/private:cargo_build_script.bzl", "get_cc_compile_args_and_env")
load("@rules_rust//rust/private:rustc.bzl", "collect_deps", "get_linker_and_args")
load("@rules_rust//rust/private:utils.bzl", "find_cc_toolchain", "find_toolchain", "transform_deps")

def _PostcompilerDepsInfo_init(**kwargs):
    return kwargs | {
        "__PostcompilerDepsInfo": True,
    }

PostcompilerDepsInfo, _ = provider(
    doc = "Information about a postcompile test dependencies",
    fields = [
        "__PostcompilerDepsInfo",
        "env",
    ],
    init = _PostcompilerDepsInfo_init,
)

def _pwd_flags_sysroot(args):
    """Prefix execroot-relative paths of known arguments with ${pwd}.

    Args:
        args (list): List of tool arguments.

    Returns:
        list: The modified argument list.
    """
    res = []
    for arg in args:
        s, opt, path = arg.partition("--sysroot=")
        if s == "" and not paths.is_absolute(path):
            res.append("{}${{pwd}}/{}".format(opt, path))
        else:
            res.append(arg)
    return res

def _pwd_flags_isystem(args):
    """Prefix execroot-relative paths of known arguments with ${pwd}.

    Args:
        args (list): List of tool arguments.

    Returns:
        list: The modified argument list.
    """
    res = []
    fix_next_arg = False
    for arg in args:
        if fix_next_arg and not paths.is_absolute(arg):
            res.append("${{pwd}}/{}".format(arg))
        else:
            res.append(arg)

        fix_next_arg = arg == "-isystem"

    return res

def _pwd_flags_bindir(args):
    """Prefix execroot-relative paths of known arguments with ${pwd}.

    Args:
        args (list): List of tool arguments.

    Returns:
        list: The modified argument list.
    """
    res = []
    fix_next_arg = False
    for arg in args:
        if fix_next_arg and not paths.is_absolute(arg):
            res.append("${{pwd}}/{}".format(arg))
        else:
            res.append(arg)

        fix_next_arg = arg == "-B"

    return res

def _pwd_flags_fsanitize_ignorelist(args):
    """Prefix execroot-relative paths of known arguments with ${pwd}.

    Args:
        args (list): List of tool arguments.

    Returns:
        list: The modified argument list.
    """
    res = []
    for arg in args:
        s, opt, path = arg.partition("-fsanitize-ignorelist=")
        if s == "" and not paths.is_absolute(path):
            res.append("{}${{pwd}}/{}".format(opt, path))
        else:
            res.append(arg)
    return res

def _pwd_flags(args):
    return _pwd_flags_fsanitize_ignorelist(_pwd_flags_isystem(_pwd_flags_bindir(_pwd_flags_sysroot(args))))

def _transform_path(path):
    return path.replace("external/", "../")

def _postcompile_deps_impl(ctx):
    rust_toolchain = find_toolchain(ctx)
    cc_toolchain, feature_configuration = find_cc_toolchain(ctx)

    env = {}
    linker, link_args, linker_env = get_linker_and_args(ctx, "bin", cc_toolchain, feature_configuration, None)

    env.update({k: _transform_path(v) for k, v in linker_env.items()})
    env["LD"] = _transform_path(linker)
    env["LDFLAGS"] = _transform_path(" ".join(_pwd_flags(link_args)))

    cc_c_args, cc_cxx_args, cc_env = get_cc_compile_args_and_env(cc_toolchain, feature_configuration)
    include = cc_env.get("INCLUDE")
    if include:
        env["INCLUDE"] = _transform_path(include)

    toolchain_tools = [rust_toolchain.all_files]

    extra_rustc_args = [
        "-C",
        "linker={}".format(env["LD"]),
        "-C",
        "link-arg=-Wl,-znostart-stop-gc",
    ]
    for flag in _pwd_flags(link_args):
        extra_rustc_args += ["-C", "link-arg={}".format(_transform_path(flag))]

    if cc_toolchain:
        toolchain_tools.append(cc_toolchain.all_files)

        env["CC"] = _transform_path(cc_common.get_tool_for_action(
            feature_configuration = feature_configuration,
            action_name = ACTION_NAMES.c_compile,
        ))
        env["CXX"] = _transform_path(cc_common.get_tool_for_action(
            feature_configuration = feature_configuration,
            action_name = ACTION_NAMES.cpp_compile,
        ))
        env["AR"] = _transform_path(cc_common.get_tool_for_action(
            feature_configuration = feature_configuration,
            action_name = ACTION_NAMES.cpp_link_static_library,
        ))

        # Populate CFLAGS and CXXFLAGS that cc-rs relies on when building from source, in particular
        # to determine the deployment target when building for apple platforms (`macosx-version-min`
        # for example, itself derived from the `macos_minimum_os` Bazel argument).
        env["CFLAGS"] = " ".join(_pwd_flags(cc_c_args))
        env["CXXFLAGS"] = " ".join(_pwd_flags(cc_cxx_args))

    is_windows = ctx.target_platform_has_constraint(ctx.attr._windows_constraint[platform_common.ConstraintValueInfo])
    if is_windows:
        rustc_wrapper = ctx.actions.declare_file(ctx.label.name + ".bat")
        ctx.actions.write(
            output = rustc_wrapper,
            content = '@"%~dp0{}" %*'.format(ctx.attr._test_runner[DefaultInfo].files_to_run.executable.short_path),
            is_executable = True,
        )
    else:
        rustc_wrapper = ctx.actions.declare_file(ctx.label.name + ".sh")
        ctx.actions.write(
            output = rustc_wrapper,
            content = '#!/bin/env bash\nexec "{}" --subst pwd=$(pwd) -- "{}" $@'.format(ctx.executable._process_wrapper.short_path, rust_toolchain.rustc.short_path),
            is_executable = True,
        )

    env.update({
        "RUSTC": rustc_wrapper.short_path,
        "TARGET": rust_toolchain.target_flag_value,
    })

    dep_info, _, _ = collect_deps(
        deps = depset(transform_deps(ctx.attr.deps)),
        proc_macro_deps = depset(transform_deps(ctx.attr.proc_macro_deps)),
        aliases = ctx.attr.aliases,
    )

    postcompile_args = {
        "direct": {},
        "search": [],
        "extra_rustc_args": extra_rustc_args,
    }

    runfiles = []

    for crate in dep_info.direct_crates.to_list():
        if hasattr(crate, "dep"):
            name = crate.name
            crate_info = crate.dep
        else:
            name = crate.name
            crate_info = crate

        postcompile_args["direct"][name] = crate_info.output.short_path
        runfiles.append(crate_info.output)

    for crate in dep_info.transitive_crates.to_list():
        postcompile_args["search"].append(paths.dirname(crate.output.short_path))
        runfiles.append(crate.output)

    out = ctx.actions.declare_file("{}--postcompile-args.json".format(ctx.label.name))
    ctx.actions.write(
        output = out,
        content = json.encode(postcompile_args),
    )

    env["POSTCOMPILE_DEPS_MANIFEST"] = out.short_path

    for target in ctx.attr.toolchains:
        if DefaultInfo in target:
            toolchain_tools.extend([
                target[DefaultInfo].files,
                target[DefaultInfo].default_runfiles.files,
            ])
        if platform_common.ToolchainInfo in target:
            all_files = getattr(target[platform_common.ToolchainInfo], "all_files", depset([]))
            if type(all_files) == "list":
                all_files = depset(all_files)
            toolchain_tools.append(all_files)

    return [
        DefaultInfo(
            files = depset([out, rustc_wrapper]),
            runfiles = ctx.runfiles(files = runfiles + [out, rustc_wrapper], transitive_files = depset([], transitive = toolchain_tools)),
        ),
        PostcompilerDepsInfo(
            env = env,
        ),
    ]

postcompile_deps = rule(
    implementation = _postcompile_deps_impl,
    attrs = {
        "deps": attr.label_list(),
        "proc_macro_deps": attr.label_list(),
        "aliases": attr.label_keyed_string_dict(),
        "_process_wrapper": attr.label(
            default = Label("@rules_rust//util/process_wrapper"),
            executable = True,
            allow_single_file = True,
            cfg = "exec",
        ),
        "_windows_constraint": attr.label(
            default = "@platforms//os:windows",
        ),
    },
    fragments = ["cpp"],
    toolchains = [
        "@rules_rust//rust:toolchain_type",
        "@bazel_tools//tools/cpp:toolchain_type",
    ],
    provides = [DefaultInfo, PostcompilerDepsInfo],
)
