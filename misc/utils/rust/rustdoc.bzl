"""Rules for generating documentation with `rustdoc` for Bazel built crates"""

load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_rust//rust/private:common.bzl", "rust_common")
load("@rules_rust//rust/private:providers.bzl", "LintsInfo")
load("@rules_rust//rust/private:rustc.bzl", "collect_deps", "collect_inputs", "construct_arguments")
load("@rules_rust//rust/private:utils.bzl", "dedent", "expand_dict_value_locations", "find_cc_toolchain", "find_toolchain")
load("//misc/utils/rust:postcompile.bzl", "PostcompilerDepsInfo")

def _init_rust_doc_info(*, crate_name, crate_version, html_out = None, json_out = None, parts_out = None):
    if not (html_out or json_out) or (html_out and json_out):
        fail("at most one of html_out or json_out must be provided")

    if parts_out and not html_out:
        fail("parts_out can only be provided with html_out")

    return {
        "html_out": html_out,
        "json_out": json_out,
        "parts_out": parts_out,
        "crate_name": crate_name,
        "crate_version": crate_version,
    }

RustDocInfo, _ = provider(
    doc = """
    Info related to rustdoc generation.
    """,
    fields = [
        "html_out",
        "json_out",
        "parts_out",
        "crate_name",
        "crate_version",
    ],
    init = _init_rust_doc_info,
)

def _strip_crate_info_output(crate_info):
    """Set the CrateInfo.output to None for a given CrateInfo provider.

    Args:
        crate_info (CrateInfo): A provider

    Returns:
        CrateInfo: A modified CrateInfo provider
    """
    return rust_common.create_crate_info(
        name = crate_info.name,
        type = crate_info.type,
        root = crate_info.root,
        srcs = crate_info.srcs,
        deps = crate_info.deps,
        proc_macro_deps = crate_info.proc_macro_deps,
        aliases = crate_info.aliases,
        # This crate info should have no output
        output = None,
        metadata = None,
        edition = crate_info.edition,
        rustc_env = crate_info.rustc_env,
        rustc_env_files = crate_info.rustc_env_files,
        is_test = crate_info.is_test,
        compile_data = crate_info.compile_data,
        compile_data_targets = crate_info.compile_data_targets,
        data = crate_info.data,
    )

def _strip_crate_info_output_doctest(crate_info):
    """Set the CrateInfo.output to None for a given CrateInfo provider.

    Args:
        crate_info (CrateInfo): A provider

    Returns:
        CrateInfo: A modified CrateInfo provider
    """
    return rust_common.create_crate_info(
        name = None,
        type = None,
        root = None,
        srcs = None,
        deps = crate_info.deps,
        proc_macro_deps = crate_info.proc_macro_deps,
        aliases = crate_info.aliases,
        # This crate info should have no output
        output = None,
        metadata = None,
        edition = None,
        rustc_env = crate_info.rustc_env,
        rustc_env_files = crate_info.rustc_env_files,
        is_test = False,
        compile_data = crate_info.compile_data,
        compile_data_targets = crate_info.compile_data_targets,
        data = crate_info.data,
    )

def _rustdoc_compile_action(
        ctx,
        toolchain,
        crate_info,
        deps = None,
        proc_macro_deps = None,
        aliases = None,
        lints_info = None,
        rustdoc_flags = []):
    """Create a struct of information needed for a `rustdoc` compile action based on crate passed to the rustdoc rule.

    Args:
        ctx (ctx): The rule's context object.
        toolchain (rust_toolchain): The currently configured `rust_toolchain`.
        crate_info (CrateInfo): The provider of the crate passed to a rustdoc rule.
        lints_info (LintsInfo, optional): The LintsInfo provider of the crate passed to the rustdoc rule.
        rustdoc_flags (list, optional): A list of `rustdoc` specific flags.
        deps (list, optional): A list of deps.
        proc_macro_deps (list, optional): A list of proc macro deps.
        aliases: (dict, optional): A set of aliases in the crate.

    Returns:
        struct: A struct of some `ctx.actions.run` arguments.
    """

    # Specify rustc flags for lints, if they were provided.
    lint_files = []
    if lints_info:
        rustdoc_flags = rustdoc_flags + lints_info.rustdoc_lint_flags
        lint_files = lint_files + lints_info.rustdoc_lint_files

    cc_toolchain, feature_configuration = find_cc_toolchain(ctx)

    dep_info, build_info, _ = collect_deps(
        deps = crate_info.deps if deps == None else deps,
        proc_macro_deps = crate_info.proc_macro_deps if proc_macro_deps == None else proc_macro_deps,
        aliases = crate_info.aliases if aliases == None else aliases,
    )

    def update_external_links(external_links, crate):
        if hasattr(crate, "dep"):
            name = crate.name
            crate_info = crate.dep
        else:
            name = crate.name
            crate_info = crate
        external_links[name] = "https://docs.rs/{name}/{version}".format(name = name, version = crate_info.version)

    external_links = {}
    if hasattr(ctx.attr, "rustdoc_map") and ctx.attr.rustdoc_map:
        for crate in dep_info.transitive_crates.to_list():
            update_external_links(external_links, crate)
        for crate in dep_info.direct_crates.to_list():
            update_external_links(external_links, crate)

    if hasattr(ctx.attr, "external_links"):
        external_links.update(ctx.attr.external_links)

    for name, link in external_links.items():
        rustdoc_flags.append("--extern-html-root-url={}={}".format(name, link))

    compile_inputs, out_dir, build_env_files, build_flags_files, linkstamp_outs, ambiguous_libs = collect_inputs(
        ctx = ctx,
        file = ctx.file,
        files = ctx.files,
        linkstamps = depset([]),
        toolchain = toolchain,
        cc_toolchain = cc_toolchain,
        feature_configuration = feature_configuration,
        crate_info = crate_info,
        dep_info = dep_info,
        build_info = build_info,
        lint_files = lint_files,
        force_depend_on_objects = False,
        include_link_flags = False,
    )

    # Since this crate is not actually producing the output described by the
    # given CrateInfo, this attribute needs to be stripped to allow the rest
    # of the rustc functionality in `construct_arguments` to avoid generating
    # arguments expecting to do so.
    rustdoc_crate_info = _strip_crate_info_output(crate_info)

    args, env = construct_arguments(
        ctx = ctx,
        attr = ctx.attr,
        file = ctx.file,
        toolchain = toolchain,
        tool_path = toolchain.rust_doc.path,
        cc_toolchain = cc_toolchain,
        feature_configuration = feature_configuration,
        crate_info = rustdoc_crate_info,
        dep_info = dep_info,
        linkstamp_outs = linkstamp_outs,
        ambiguous_libs = ambiguous_libs,
        output_hash = None,
        rust_flags = rustdoc_flags,
        out_dir = out_dir,
        build_env_files = build_env_files,
        build_flags_files = build_flags_files,
        emit = [],
        remap_path_prefix = None,
        add_flags_for_binary = True,
        include_link_flags = False,
        force_depend_on_objects = False,
        skip_expanding_rustc_env = True,
    )

    data_paths = depset(direct = getattr(ctx.attr, "data", []), transitive = [crate_info.compile_data_targets]).to_list()
    env.update(expand_dict_value_locations(
        ctx,
        ctx.attr.rustdoc_env,
        data_paths,
        {},
    ))

    # Create the combined inputs including HTML customization files
    all_inputs = depset([crate_info.output], transitive = [compile_inputs, depset(ctx.files.rustdoc_env_files)])

    for build_env_file in ctx.files.rustdoc_env_files:
        args.process_wrapper_flags.add("--env-file", build_env_file)

    return struct(
        executable = ctx.executable._process_wrapper,
        inputs = all_inputs,
        env = env,
        arguments = args.all,
        tools = [toolchain.rust_doc],
    )

def _rustdoc_impl(ctx):
    """The implementation of the `rust_doc` rule

    Args:
        ctx (ctx): The rule's context object
    """
    crate = ctx.attr.crate
    crate_info = crate[rust_common.crate_info]
    lints_info = crate[LintsInfo] if LintsInfo in crate else None

    html_out = None
    json_out = None
    parts_out = None

    outputs = []

    # Add the current crate as an extern for the compile action
    rustdoc_flags = [
        "-Zunstable-options",
        "--output-format",
        ctx.attr.output_format,
        "--extern",
        "{}={}".format(crate_info.name, crate_info.output.path),
        "--crate-version",
        crate_info.version,
    ]

    if ctx.attr.output_format == "html":
        html_out = ctx.actions.declare_directory("{}.rustdoc".format(ctx.label.name))
        parts_out = ctx.actions.declare_directory("{}.rustdoc.parts".format(ctx.label.name))
        rustdoc_flags.extend(["--out-dir", html_out.path, "--parts-out-dir", parts_out.path])
        outputs.extend([html_out, parts_out])
    elif ctx.attr.output_format == "json":
        json_out = ctx.actions.declare_file("{}.rustdoc.json".format(ctx.label.name))
        outputs.append(json_out)

    rustdoc_flags.extend(ctx.attr.rustdoc_flags)

    if ctx.attr.include_features:
        rustdoc_flags.extend(["--cfg=feature=\"{}\"".format(feature) for feature in crate_info.crate_features])

    action = _rustdoc_compile_action(
        ctx = ctx,
        toolchain = find_toolchain(ctx),
        crate_info = crate_info,
        lints_info = lints_info,
        rustdoc_flags = rustdoc_flags,
    )

    args = ctx.actions.args()
    args.add("--rustdoc", action.executable)
    if json_out:
        args.add("--json-out", json_out.path)
    args.add("--")

    ctx.actions.run(
        mnemonic = "Rustdoc",
        progress_message = "Generating Rustdoc for {}".format(crate.label),
        outputs = outputs,
        executable = ctx.executable._rustdoc_wrapper,
        inputs = depset([action.executable], transitive = [action.inputs]),
        env = action.env | {
            "RUSTC_BOOTSTRAP": "1",
        },
        arguments = [args] + action.arguments,
        tools = action.tools,
    )

    return [
        DefaultInfo(
            files = depset(outputs),
        ),
        RustDocInfo(
            crate_name = crate_info.name,
            crate_version = crate_info.version,
            json_out = json_out,
            html_out = html_out,
            parts_out = parts_out,
        ),
    ]

rustdoc = rule(
    doc = dedent("""\
    Generates code documentation.

    Example:
    Suppose you have the following directory structure for a Rust library crate:

    ```
    [workspace]/
        WORKSPACE
        hello_lib/
            BUILD
            src/
                lib.rs
    ```

    To build [`rustdoc`][rustdoc] documentation for the `hello_lib` crate, define \
    a `rust_doc` rule that depends on the the `hello_lib` `rust_library` target:

    [rustdoc]: https://doc.rust-lang.org/book/documentation.html

    ```python
    package(default_visibility = ["//visibility:public"])

    load("@rules_rust//rust:defs.bzl", "rust_library", "rust_doc")

    rust_library(
        name = "hello_lib",
        srcs = ["src/lib.rs"],
    )

    rust_doc(
        name = "hello_lib_doc",
        crate = ":hello_lib",
    )
    ```

    Running `bazel build //hello_lib:hello_lib_doc` will build a zip file containing \
    the documentation for the `hello_lib` library crate generated by `rustdoc`.
    """),
    implementation = _rustdoc_impl,
    attrs = {
        "crate": attr.label(
            doc = (
                "The label of the target to generate code documentation for.\n" +
                "\n" +
                "`rust_doc` can generate HTML code documentation for the source files of " +
                "`rust_library` or `rust_binary` targets."
            ),
            providers = [rust_common.crate_info],
            mandatory = True,
        ),
        "rustdoc_map": attr.bool(default = True),
        "external_links": attr.string_dict(),
        "output_format": attr.string(
            default = "html",
            values = ["json", "html"],
        ),
        "rustdoc_flags": attr.string_list(
            doc = dedent("""\
                List of flags passed to `rustdoc`.

                These strings are subject to Make variable expansion for predefined
                source/output path variables like `$location`, `$execpath`, and
                `$rootpath`. This expansion is useful if you wish to pass a generated
                file of arguments to rustc: `@$(location //package:target)`.
            """),
        ),
        "rustdoc_env": attr.string_dict(
            doc = dedent("""\
                Dictionary of additional `"key": "value"` environment variables to set for rustdoc.

                rust_test()/rust_binary() rules can use $(rootpath //package:target) to pass in the
                location of a generated file or external tool. Cargo build scripts that wish to
                expand locations should use cargo_build_script()'s build_script_env argument instead,
                as build scripts are run in a different environment - see cargo_build_script()'s
                documentation for more.
            """),
        ),
        "rustdoc_env_files": attr.label_list(
            doc = dedent("""\
                Files containing additional environment variables to set for rustdoc.

                These files should  contain a single variable per line, of format
                `NAME=value`, and newlines may be included in a value by ending a
                line with a trailing back-slash (`\\\\`).

                The order that these files will be processed is unspecified, so
                multiple definitions of a particular variable are discouraged.

                Note that the variables here are subject to
                [workspace status](https://docs.bazel.build/versions/main/user-manual.html#workspace_status)
                stamping should the `stamp` attribute be enabled. Stamp variables
                should be wrapped in brackets in order to be resolved. E.g.
                `NAME={WORKSPACE_STATUS_VARIABLE}`.
            """),
            allow_files = True,
        ),
        "include_features": attr.bool(
            doc = "Include the features defined by `crate_features` when building the doc tests.",
            default = True,
        ),
        "_rustdoc_wrapper": attr.label(
            default = Label("//misc/utils/rust/doc/wrapper"),
            cfg = "exec",
            executable = True,
        ),
        "_error_format": attr.label(
            default = Label("@rules_rust//rust/settings:error_format"),
        ),
        "_process_wrapper": attr.label(
            doc = "A process wrapper for running rustdoc on all platforms",
            default = Label("@rules_rust//util/process_wrapper"),
            executable = True,
            allow_single_file = True,
            cfg = "exec",
        ),
    },
    fragments = ["cpp"],
    toolchains = [
        str(Label("@rules_rust//rust:toolchain_type")),
        "@bazel_tools//tools/cpp:toolchain_type",
    ],
)

def _are_linkstamps_supported(feature_configuration):
    # Are linkstamps supported by the C++ toolchain?
    return (cc_common.is_enabled(feature_configuration = feature_configuration, feature_name = "linkstamps") and
            # Is Bazel recent enough to support Starlark linkstamps?
            hasattr(cc_common, "register_linkstamp_compile_action"))

def _rustc_doctest_compile_action(
        ctx,
        toolchain,
        crate_info,
        deps = None,
        proc_macro_deps = None,
        aliases = None,
        lints_info = None,
        rustc_flags = []):
    """Create a struct of information needed for a `rustdoc` compile action based on crate passed to the rustdoc_test rule.

    Args:
        ctx (ctx): The rule's context object.
        toolchain (rust_toolchain): The currently configured `rust_toolchain`.
        crate_info (CrateInfo): The provider of the crate passed to a rustdoc rule.
        lints_info (LintsInfo, optional): The LintsInfo provider of the crate passed to the rustdoc rule.
        rustc_flags (list, optional): A list of `rustc` specific flags.
        deps (list, optional): A list of deps.
        proc_macro_deps (list, optional): A list of proc macro deps.
        aliases: (dict, optional): A set of aliases in the crate.

    Returns:
        struct: A struct of some `ctx.actions.run` arguments.
    """

    # Specify rustc flags for lints, if they were provided.
    lint_files = []
    if lints_info:
        rustc_flags = rustc_flags + lints_info.rustc_lint_flags
        lint_files = lint_files + lints_info.rustc_lint_files

    dep_info, build_info, linkstamps = collect_deps(
        deps = crate_info.deps if deps == None else deps,
        proc_macro_deps = crate_info.proc_macro_deps if proc_macro_deps == None else proc_macro_deps,
        aliases = crate_info.aliases if aliases == None else aliases,
    )

    cc_toolchain, feature_configuration = find_cc_toolchain(ctx, [])
    if not _are_linkstamps_supported(feature_configuration):
        linkstamps = depset([])

    compile_inputs, out_dir, build_env_files, build_flags_files, linkstamp_outs, ambiguous_libs = collect_inputs(
        ctx = ctx,
        file = ctx.file,
        files = ctx.files,
        linkstamps = linkstamps,
        toolchain = toolchain,
        cc_toolchain = cc_toolchain,
        feature_configuration = feature_configuration,
        crate_info = crate_info,
        dep_info = dep_info,
        build_info = build_info,
        lint_files = lint_files,
        force_depend_on_objects = True,
        include_link_flags = True,
    )

    args, env = construct_arguments(
        ctx = ctx,
        attr = ctx.attr,
        file = ctx.file,
        toolchain = toolchain,
        tool_path = toolchain.rustc.path,
        cc_toolchain = cc_toolchain,
        feature_configuration = feature_configuration,
        crate_info = _strip_crate_info_output_doctest(crate_info),
        dep_info = dep_info,
        linkstamp_outs = linkstamp_outs,
        ambiguous_libs = ambiguous_libs,
        output_hash = None,
        rust_flags = rustc_flags,
        out_dir = out_dir,
        build_env_files = build_env_files,
        build_flags_files = build_flags_files,
        emit = [],
        remap_path_prefix = None,
        add_flags_for_binary = True,
        include_link_flags = False,
        force_depend_on_objects = True,
        skip_expanding_rustc_env = True,
    )

    data_paths = depset(direct = getattr(ctx.attr, "data", []), transitive = [crate_info.compile_data_targets]).to_list()
    env.update(expand_dict_value_locations(
        ctx,
        ctx.attr.rustdoc_env,
        data_paths,
        {},
    ))

    # Create the combined inputs including HTML customization files
    all_inputs = depset([crate_info.output], transitive = [compile_inputs, depset(ctx.files.rustc_env_files)])

    for build_env_file in ctx.files.rustc_env_files:
        args.process_wrapper_flags.add("--env-file", build_env_file)

    return struct(
        executable = ctx.executable._process_wrapper,
        inputs = all_inputs,
        env = env,
        arguments = args.all,
        tools = [toolchain.rustc],
    )

def _rustdoc_test_impl(ctx):
    """The implementation of the `rust_doc` rule

    Args:
        ctx (ctx): The rule's context object
    """

    crate = ctx.attr.crate
    crate_info = crate[rust_common.crate_info]
    lints_info = crate[LintsInfo] if LintsInfo in crate else None

    extract_out = ctx.actions.declare_file("{}.rustdoc_tests.jsonl".format(ctx.label.name))

    # Add the current crate as an extern for the compile action
    rust_flags = [
        "--extern",
        "{}={}".format(crate_info.name, crate_info.output.path),
    ]

    if ctx.attr.include_features:
        rust_flags.extend(["--cfg=feature=\"{}\"".format(feature) for feature in crate_info.crate_features])

    action = _rustdoc_compile_action(
        ctx = ctx,
        toolchain = find_toolchain(ctx),
        crate_info = crate_info,
        lints_info = lints_info,
        rustdoc_flags = ["-Zunstable-options"] + rust_flags + ctx.attr.rustdoc_flags,
        deps = depset(ctx.attr.deps),
        proc_macro_deps = depset(ctx.attr.proc_macro_deps),
        aliases = ctx.attr.aliases,
    )

    args = ctx.actions.args()
    args.add("--rustdoc", action.executable)
    args.add("--test-out", extract_out.path)
    args.add("--")

    ctx.actions.run(
        mnemonic = "RustdocTestExtract",
        progress_message = "Extracting Rustdoc Tests for {}".format(crate.label),
        outputs = [extract_out],
        executable = ctx.executable._rustdoc_wrapper,
        inputs = depset([action.executable], transitive = [action.inputs]),
        env = action.env | {
            "RUSTC_BOOTSTRAP": "1",
        },
        arguments = [args] + action.arguments,
        tools = action.tools,
    )

    action = _rustc_doctest_compile_action(
        ctx = ctx,
        toolchain = find_toolchain(ctx),
        crate_info = crate_info,
        lints_info = lints_info,
        rustc_flags = rust_flags,
        deps = depset(ctx.attr.deps),
        proc_macro_deps = depset(ctx.attr.proc_macro_deps),
        aliases = ctx.attr.aliases,
    )

    build_out = ctx.actions.declare_directory("{}.rustdoc_tests".format(ctx.label.name))

    args = ctx.actions.args()
    args.add("--rustc", action.executable)
    args.add("--edition", crate_info.edition or "2015")
    args.add("--out-dir", build_out.path)
    args.add("--extracted-tests", extract_out)
    args.add("--")

    ctx.actions.run(
        mnemonic = "RustdocTestBuild",
        progress_message = "Building Rustdoc Tests for {}".format(crate.label),
        outputs = [build_out],
        executable = ctx.executable._rust_doctest_builder,
        inputs = depset([action.executable, extract_out], transitive = [action.inputs]),
        env = action.env,
        arguments = [args] + action.arguments,
        tools = action.tools,
    )

    nextest_config = ctx.attr._nextest_config[DefaultInfo].files.to_list()[0]

    env = expand_dict_value_locations(
        ctx,
        ctx.attr.env,
        ctx.attr.data,
        {},
    )

    env.update({
        "RUNNER_CRATE": crate_info.name,
        "RUNNER_DOCTEST_OUT": build_out.short_path,
        "RUNNER_CONFIG": nextest_config.short_path,
        "RUNNER_PROFILE": ctx.attr._nextest_profile[BuildSettingInfo].value,
    })

    is_windows = ctx.target_platform_has_constraint(ctx.attr._windows_constraint[platform_common.ConstraintValueInfo])
    if is_windows:
        wrapper_script = ctx.actions.declare_file(ctx.label.name + ".bat")
        ctx.actions.write(
            output = wrapper_script,
            content = '@"{}" --subst "pwd=${{pwd}}" -- "{}" %*'.format(
                ctx.executable._process_wrapper.short_path,
                ctx.executable._rust_doctest_runner.short_path,
            ),
            is_executable = True,
        )
    else:
        wrapper_script = ctx.actions.declare_file(ctx.label.name + ".sh")
        ctx.actions.write(
            output = wrapper_script,
            content = '#!/usr/bin/env bash\nexec "{}" --subst \'pwd=${{pwd}}\' -- "{}" $@'.format(
                ctx.executable._process_wrapper.short_path,
                ctx.executable._rust_doctest_runner.short_path,
            ),
            is_executable = True,
        )

    runfiles = []
    for data in ctx.attr.data:
        if PostcompilerDepsInfo in data:
            env.update(data[PostcompilerDepsInfo].env)

        runfiles.append(data[DefaultInfo].files)

    runfiles = ctx.runfiles(files = [build_out, nextest_config, ctx.executable._process_wrapper, ctx.executable._rust_doctest_runner], transitive_files = depset([], transitive = runfiles))
    runfiles = runfiles.merge(ctx.attr._rust_doctest_runner[DefaultInfo].default_runfiles)
    runfiles = runfiles.merge_all([dep[DefaultInfo].default_runfiles for dep in ctx.attr.data])

    return [
        DefaultInfo(
            files = depset([build_out, wrapper_script]),
            executable = wrapper_script,
            runfiles = runfiles,
        ),
        RunEnvironmentInfo(
            environment = env,
        ),
    ]

rustdoc_test = rule(
    implementation = _rustdoc_test_impl,
    test = True,
    attrs = {
        "crate": attr.label(
            providers = [rust_common.crate_info],
            mandatory = True,
        ),
        "rustdoc_flags": attr.string_list(
            doc = dedent("""\
                List of flags passed to `rustdoc`.

                These strings are subject to Make variable expansion for predefined
                source/output path variables like `$location`, `$execpath`, and
                `$rootpath`. This expansion is useful if you wish to pass a generated
                file of arguments to rustc: `@$(location //package:target)`.
            """),
        ),
        "rustc_flags": attr.string_list(
            doc = dedent("""\
                List of flags passed to `rustc`.

                These strings are subject to Make variable expansion for predefined
                source/output path variables like `$location`, `$execpath`, and
                `$rootpath`. This expansion is useful if you wish to pass a generated
                file of arguments to rustc: `@$(location //package:target)`.
            """),
        ),
        "deps": attr.label_list(
            doc = "Additional deps needed by the doctests.",
        ),
        "proc_macro_deps": attr.label_list(
            doc = "Additional proc-macro deps needed by the doctests.",
            cfg = "exec",
        ),
        "aliases": attr.label_keyed_string_dict(),
        "include_features": attr.bool(
            doc = "Include the features defined by `crate_features` when building the doc tests.",
            default = True,
        ),
        "rustdoc_env": attr.string_dict(
            doc = dedent("""\
                Dictionary of additional `"key": "value"` environment variables to set for rustdoc.

                rust_test()/rust_binary() rules can use $(rootpath //package:target) to pass in the
                location of a generated file or external tool. Cargo build scripts that wish to
                expand locations should use cargo_build_script()'s build_script_env argument instead,
                as build scripts are run in a different environment - see cargo_build_script()'s
                documentation for more.
            """),
        ),
        "rustdoc_env_files": attr.label_list(
            doc = dedent("""\
                Files containing additional environment variables to set for rustdoc.

                These files should  contain a single variable per line, of format
                `NAME=value`, and newlines may be included in a value by ending a
                line with a trailing back-slash (`\\\\`).

                The order that these files will be processed is unspecified, so
                multiple definitions of a particular variable are discouraged.

                Note that the variables here are subject to
                [workspace status](https://docs.bazel.build/versions/main/user-manual.html#workspace_status)
                stamping should the `stamp` attribute be enabled. Stamp variables
                should be wrapped in brackets in order to be resolved. E.g.
                `NAME={WORKSPACE_STATUS_VARIABLE}`.
            """),
            allow_files = True,
        ),
        "rustc_env": attr.string_dict(
            doc = dedent("""\
                Dictionary of additional `"key": "value"` environment variables to set for rustdoc.

                rust_test()/rust_binary() rules can use $(rootpath //package:target) to pass in the
                location of a generated file or external tool. Cargo build scripts that wish to
                expand locations should use cargo_build_script()'s build_script_env argument instead,
                as build scripts are run in a different environment - see cargo_build_script()'s
                documentation for more.
            """),
        ),
        "rustc_env_files": attr.label_list(
            doc = dedent("""\
                Files containing additional environment variables to set for rustdoc.

                These files should  contain a single variable per line, of format
                `NAME=value`, and newlines may be included in a value by ending a
                line with a trailing back-slash (`\\\\`).

                The order that these files will be processed is unspecified, so
                multiple definitions of a particular variable are discouraged.

                Note that the variables here are subject to
                [workspace status](https://docs.bazel.build/versions/main/user-manual.html#workspace_status)
                stamping should the `stamp` attribute be enabled. Stamp variables
                should be wrapped in brackets in order to be resolved. E.g.
                `NAME={WORKSPACE_STATUS_VARIABLE}`.
            """),
            allow_files = True,
        ),
        "env": attr.string_dict(),
        "data": attr.label_list(
            allow_files = True,
        ),
        "_error_format": attr.label(
            default = Label("@rules_rust//rust/settings:error_format"),
        ),
        "_rustdoc_wrapper": attr.label(
            default = Label("//misc/utils/rust/doc/wrapper"),
            cfg = "exec",
            executable = True,
        ),
        "_rust_doctest_builder": attr.label(
            default = Label("//misc/utils/rust/doc/test/builder"),
            cfg = "exec",
            executable = True,
        ),
        "_rust_doctest_runner": attr.label(
            default = Label("//misc/utils/rust/doc/test/runner"),
            cfg = "exec",
            executable = True,
        ),
        "_process_wrapper": attr.label(
            doc = "A process wrapper for running rustdoc on all platforms",
            default = Label("@rules_rust//util/process_wrapper"),
            executable = True,
            allow_single_file = True,
            cfg = "exec",
        ),
        "_nextest_profile": attr.label(mandatory = False, default = "//settings:test_profile"),
        "_nextest_config": attr.label(
            default = "//:.config/nextest.toml",
            allow_single_file = True,
        ),
        "_windows_constraint": attr.label(
            default = "@platforms//os:windows",
        ),
    },
    fragments = ["cpp"],
    toolchains = [
        str(Label("@rules_rust//rust:toolchain_type")),
        "@bazel_tools//tools/cpp:toolchain_type",
    ],
)

def _rustdoc_merge_impl(ctx):
    output = ctx.actions.declare_directory("{}.rustdoc_merge".format(ctx.label.name))

    toolchain = find_toolchain(ctx)

    deps = [toolchain.rust_doc]
    entries = []
    for target in ctx.attr.targets:
        info = target[RustDocInfo]
        deps.extend([item for item in [info.html_out, info.json_out, info.parts_out] if item])
        entries.append({
            "html_out": info.html_out.path if info.html_out else None,
            "json_out": info.json_out.path if info.json_out else None,
            "parts_out": info.parts_out.path if info.parts_out else None,
            "crate_name": info.crate_name,
            "crate_version": info.crate_version,
        })

    manifest = ctx.actions.declare_file("{}.rustdoc_merge.manifest.json".format(ctx.label.name))
    ctx.actions.write(
        output = manifest,
        content = json.encode({
            "entries": entries,
        }),
    )

    deps.append(manifest)

    args = ctx.actions.args()

    args.add("--rustdoc", toolchain.rust_doc.path)
    args.add("--manifest", manifest.path)
    args.add("--output", output.path)

    ctx.actions.run(
        mnemonic = "RustdocMerge",
        progress_message = "Merging Rustdoc for {}".format(ctx.label),
        outputs = [output],
        executable = ctx.executable._rustdoc_merger,
        inputs = depset(deps),
        env = {},
        arguments = [args],
        tools = [toolchain.rust_doc],
    )

    return [
        DefaultInfo(
            files = depset([output]),
        ),
    ]

rustdoc_merge = rule(
    implementation = _rustdoc_merge_impl,
    attrs = {
        "targets": attr.label_list(
            providers = [RustDocInfo],
        ),
        "_rustdoc_merger": attr.label(
            default = "//misc/utils/rust/doc/merger",
            cfg = "exec",
            executable = True,
        ),
    },
    fragments = ["cpp"],
    toolchains = [
        str(Label("@rules_rust//rust:toolchain_type")),
        "@bazel_tools//tools/cpp:toolchain_type",
    ],
)
