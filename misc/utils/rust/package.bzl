"""
Helper scripts for setting up rust targets.
"""

load("@cargo_vendor//:defs.bzl", "all_crate_deps", "crate_features", "crate_version", dep_aliases = "aliases")
load("@rules_rust//cargo:defs.bzl", "cargo_build_script", "cargo_toml_env_vars", "extract_cargo_lints")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_proc_macro", "rustfmt_test")
load("//misc/utils/rust:clippy.bzl", "rust_clippy", "rust_clippy_test")
load("//misc/utils/rust:rust_analyzer.bzl", "rust_analyzer_info")
load("//misc/utils/rust:rustdoc.bzl", "rustdoc", "rustdoc_test")
load("//misc/utils/rust:sync_readme.bzl", "sync_readme", "sync_readme_test")
load("//misc/utils/rust:test.bzl", "nextest_test")

gc_arg = select({
    "@platforms//os:linux": ["-Clink-arg=-Wl,-znostart-stop-gc"],
    "//conditions:default": [],
})

def scuffle_package(
        crate_name,
        name = None,
        version = None,
        features = None,
        crate_type = "rlib",
        srcs = None,
        visibility = None,
        aliases = None,
        deps = None,
        proc_macro_deps = None,
        compile_data = None,
        tags = None,
        test = None,
        readme = None,
        extra_target_kwargs = None,
        target_compatible_with = None,
        rustc_flags = None,
        rustc_env = None,
        stable_as_nightly = None):
    """Creates a rust_library and corresponding rust_test target.

    Args:
        crate_name: Name of the crate.
        name: Name of the target
        version: Version of the crate.
        features: A set of features this crate has
        crate_type: The type of crate to build: Default "rlib"
        srcs: Source files. Defaults to glob(["src/**/*.rs"]) if not provided
        visibility: Visibility for the library target. Defaults to ["//visibility:public"]
        aliases: Dependency aliases
        deps: Additional deps to add.
        proc_macro_deps: Additional proc macro deps to add.
        compile_data: Data to include during compile time
        tags: Additional tags to add to the package
        test: A config for testing this library.
        readme: The readme file, if set to `False` disable sync-readme
        extra_target_kwargs: additional kwargs to pass to the target.
        target_compatible_with: The compatability constraint of the target.
        rustc_flags: Additional rustc flags to add to the build.
        rustc_env: Additional env vars to add to rustc.
        stable_as_nightly: If we should use nightly mode.
    """

    package_name = native.package_name()

    # Set defaults
    if srcs == None:
        srcs = native.glob(["src/**/*.rs"])
    if visibility == None:
        visibility = ["//visibility:public"]
    if deps == None:
        deps = []
    if aliases == None:
        aliases = {}
    if proc_macro_deps == None:
        proc_macro_deps = []
    if compile_data == None:
        compile_data = []
    if features == None:
        features = crate_features(package_name = package_name, all = True)
    if tags == None:
        tags = []
    if test == None:
        test = {} if crate_type == "rlib" else False
    if extra_target_kwargs == None:
        extra_target_kwargs = {}
    if version == None:
        version = crate_version()
    if readme == None:
        readme = ":README.md"
    if target_compatible_with == None:
        target_compatible_with = []
    if stable_as_nightly == None:
        stable_as_nightly = False
    if rustc_flags == None:
        rustc_flags = []
    if rustc_env == None:
        rustc_env = {}

    NAME_MAPPINGS = {
        "rlib": "lib",
        "bin": "bin",
        "proc_macro": "macro",
    }

    if crate_type not in NAME_MAPPINGS:
        fail("crate_type must be one of: %s" % [kind for kind in NAME_MAPPINGS.keys()])

    name = package_name.split("/")[-1] if name == None else name

    cargo_toml_env_vars(
        name = name + "_cargo_toml_env",
        src = ":Cargo.toml",
        workspace = "//:Cargo.toml",
        tags = ["manual"],
        target_compatible_with = target_compatible_with,
        visibility = ["//visibility:private"],
    )

    extract_cargo_lints(
        name = name + "_cargo_toml_lints",
        manifest = ":Cargo.toml",
        workspace = "//:Cargo.toml",
        tags = ["manual"],
        target_compatible_with = target_compatible_with,
        visibility = ["//visibility:private"],
    )

    if stable_as_nightly:
        rustc_env = rustc_env | {
            "RUSTC_BOOTSTRAP": "1",
        }

    colon_name = ":" + name

    normal_deps = all_crate_deps(normal = True, package_name = package_name, features = features) + deps + ["@rules_rust//rust/runfiles"]
    normal_proc_macro_deps = all_crate_deps(proc_macro = True, package_name = package_name, features = features) + proc_macro_deps
    aliases = aliases | dep_aliases(package_name = package_name, features = features)

    rustc_flags_combined = rustc_flags + [
        "--cfg=bazel_runfiles",
    ] + gc_arg

    kwargs = extra_target_kwargs | dict(
        name = name,
        crate_name = crate_name.replace("-", "_"),
        lint_config = colon_name + "_cargo_toml_lints",
        srcs = srcs,
        crate_features = features.select(),
        aliases = aliases,
        deps = normal_deps,
        proc_macro_deps = normal_proc_macro_deps,
        visibility = visibility,
        compile_data = compile_data,
        version = version,
        tags = tags,
        rustc_flags = rustc_flags_combined,
        rustc_env = rustc_env,
        rustc_env_files = [colon_name + "_cargo_toml_env"],
        target_compatible_with = target_compatible_with,
    )

    # Create the library target
    if crate_type == "rlib":
        rust_library(**kwargs)
    elif crate_type == "proc_macro":
        rust_proc_macro(**kwargs)
    elif crate_type == "bin":
        rust_binary(**kwargs)

    rust_targets = [colon_name]
    test_target_compatible_with = target_compatible_with

    if test != False:
        test_deps = test.get("deps", [])[:]
        test_proc_macro_deps = test.get("proc_macro_deps", [])[:]
        test_env = test.get("env", {}) | {}
        test_data = test.get("data", [])[:]
        test_insta = test.get("insta", False)
        test_tags = test.get("tags", [])[:]
        test_target_compatible_with = test.get("target_compatible_with", []) + target_compatible_with

        if crate_type == "proc_macro":
            test_proc_macro_deps.append(colon_name)
        else:
            test_deps.append(colon_name)

        if test_insta:
            test_data += native.glob(["src/**/*"])

        aliases = aliases | dep_aliases(package_name = package_name, features = features, normal = True, normal_dev = True, proc_macro = True, proc_macro_dev = True)
        all_test_deps = all_crate_deps(normal = True, normal_dev = True, package_name = package_name, features = features) + deps + test_deps + ["@rules_rust//rust/runfiles"]
        all_test_proc_macro_deps = all_crate_deps(proc_macro = True, proc_macro_dev = True, package_name = package_name, features = features) + proc_macro_deps + test_proc_macro_deps

        nextest_test(
            name = name + "_test",
            insta = test_insta,
            data = test_data,
            compile_data = ["//settings:test_rustc_flags"],
            env = test_env,
            tags = test_tags,
            crate = colon_name,
            aliases = aliases,
            deps = all_test_deps,
            proc_macro_deps = all_test_proc_macro_deps,
            crate_features = features.select(),
            rustc_env_files = [colon_name + "_cargo_toml_env"],
            rustc_flags = rustc_flags_combined + [
                "--cfg=coverage_nightly",
                "@$(location //settings:test_rustc_flags)",
            ],
            rustc_env = rustc_env | {
                "RUSTC_BOOTSTRAP": "1",
            },
            # Needs to be marked as not testonly because the rust_clippy
            # rule depends on this, which we use to generate clippy suggestions
            testonly = False,
            visibility = ["//visibility:private"],
            target_compatible_with = test_target_compatible_with,
        )

        rustdoc_test(
            name = name + "_doc_test",
            crate = colon_name,
            deps = all_test_deps,
            aliases = aliases,
            proc_macro_deps = all_test_proc_macro_deps,
            data = test_data,
            env = test_env,
            tags = test_tags,
            rustc_env_files = [colon_name + "_cargo_toml_env"],
            rustc_flags = rustc_flags_combined,
            rustc_env = rustc_env,
            # Needs to be marked as not testonly because the rust_clippy
            # rule depends on this, which we use to generate clippy suggestions
            testonly = False,
            target_compatible_with = test_target_compatible_with,
        )

        rust_targets.append(colon_name + "_test")

    rust_clippy(
        name = name + "_clippy",
        targets = rust_targets,
        clippy_flags = rustc_flags_combined,
        clippy_env = rustc_env,
        visibility = ["//visibility:private"],
        target_compatible_with = test_target_compatible_with,
    )

    rust_clippy_test(
        name = name + "_clippy_test",
        targets = [colon_name + "_clippy"],
        visibility = ["//visibility:private"],
        target_compatible_with = test_target_compatible_with,
    )

    rustfmt_test(
        name = name + "_fmt_test",
        targets = rust_targets,
        target_compatible_with = test_target_compatible_with,
    )

    rustdoc_flags = [
        "-Dwarnings",
        "-Zunstable-options",
        "--cfg=docsrs",
        "--sort-modules-by-appearance",
        "--generate-link-to-definition",
        "--enable-index-page",
    ]

    if crate_type == "bin":
        rustdoc_flags.extend([
            "--document-private-items",
            "--document-hidden-items",
        ])

    rustdoc(
        name = name + "_doc",
        crate = colon_name,
        rustdoc_env_files = [colon_name + "_cargo_toml_env"],
        rustdoc_flags = rustdoc_flags,
        visibility = visibility,
        target_compatible_with = target_compatible_with,
    )

    rustdoc(
        name = name + "_doc_json",
        crate = colon_name,
        output_format = "json",
        rustdoc_env_files = [colon_name + "_cargo_toml_env"],
        rustdoc_flags = [
            "-Zunstable-options",
            "--cap-lints=allow",
            "--cfg=docsrs",
            "--sort-modules-by-appearance",
            "--document-private-items",
            "--document-hidden-items",
        ],
        visibility = visibility,
        target_compatible_with = target_compatible_with,
    )

    rust_analyzer_info(
        name = name + "_rust_analyzer",
        crate = colon_name,
        test = (colon_name + "_test") if test != False else None,
        doc_test = (colon_name + "_doc_test") if test != False else None,
        clippy = colon_name + "_clippy",
        target_compatible_with = test_target_compatible_with,
    )

    if readme != False:
        sync_readme(
            name = name + "_sync_readme",
            readme = readme,
            cargo_manifest = ":Cargo.toml",
            rustdoc = colon_name + "_doc_json",
            target_compatible_with = target_compatible_with,
        )

        sync_readme_test(
            name = name + "_sync_readme_test",
            sync_readme = colon_name + "_sync_readme",
            target_compatible_with = target_compatible_with,
        )

def scuffle_test(
        deps = None,
        proc_macro_deps = None,
        env = None,
        data = None,
        insta = False,
        tags = None,
        target_compatible_with = None):
    """Helper function to add additional typed testing values.

    Returns:
        A dict with the provided and default values.
    Args:
        deps: Test only dependencies.
        proc_macro_deps: Test only proc-macro deps.
        env: Additional env to add to the test.
        data: Additional data needed by the test.
        insta: If the test needs to work with insta snapshots.
        tags: Additional tags to add to the test.
        target_compatible_with: The compatability constraint of the target.
    """
    if deps == None:
        deps = []
    if proc_macro_deps == None:
        proc_macro_deps = []
    if env == None:
        env = {}
    if data == None:
        data = []
    if tags == None:
        tags = []
    if target_compatible_with == None:
        target_compatible_with = []

    return {
        "deps": deps,
        "proc_macro_deps": proc_macro_deps,
        "env": env,
        "data": data,
        "insta": insta,
        "tags": tags,
        "target_compatible_with": target_compatible_with,
    }

def scuffle_build_script(
        name,
        features = None,
        srcs = None,
        visibility = None,
        aliases = None,
        deps = None,
        proc_macro_deps = None,
        data = None,
        env = None,
        tools = None,
        target_compatible_with = None):
    """Creates a cargo build script

    Args:
        name: Name of the target.
        features: A set of features this crate has
        srcs: Source files. Defaults to glob(["src/**/*.rs"]) if not provided
        visibility: Visibility for the library target. Defaults to ["//visibility:private"]
        aliases: Dependency aliases
        deps: Additional deps to add.
        proc_macro_deps: Additional proc macro deps to add.
        data: Data to include during compile time
        env: Additional env variables to add when running the script.
        tools: A list of tools needed by the script.
        target_compatible_with: The compatability constraint of the target.
    """

    package_name = native.package_name()

    # Set defaults
    if srcs == None:
        srcs = ["build.rs"]
    if visibility == None:
        visibility = ["//visibility:private"]
    if deps == None:
        deps = []
    if proc_macro_deps == None:
        proc_macro_deps = []
    if aliases == None:
        aliases = {}
    if data == None:
        data = []
    if features == None:
        features = crate_features(package_name = package_name)
    if env == None:
        env = {}
    if tools == None:
        tools = []
    if target_compatible_with == None:
        target_compatible_with = []

    cargo_build_script(
        name = name,
        srcs = srcs,
        crate_features = features.select(),
        aliases = aliases | dep_aliases(package_name = package_name, features = features, build = True, build_proc_macro = True),
        deps = all_crate_deps(package_name = package_name, features = features, build = True) + deps + ["@rules_rust//rust/runfiles"],
        proc_macro_deps = all_crate_deps(package_name = package_name, features = features, build_proc_macro = True) + proc_macro_deps,
        visibility = visibility,
        data = data,
        compile_data = data,
        build_script_env = env,
        tools = tools,
        rustc_flags = [
            "--cfg=bazel_runfiles",
        ] + gc_arg,
        target_compatible_with = target_compatible_with,
    )

def scuffle_example(
        name,
        crate = None,
        features = None,
        srcs = None,
        visibility = None,
        aliases = None,
        deps = None,
        proc_macro_deps = None,
        data = None,
        target_compatible_with = None):
    """Creates a cargo build script

    Args:
        name: Name of the target.
        crate: The crate to build the example for.
        features: A set of features this crate has
        srcs: Source files. Defaults to glob(["src/**/*.rs"]) if not provided
        visibility: Visibility for the library target. Defaults to ["//visibility:private"]
        aliases: Dependency aliases
        deps: Additional deps to add.
        proc_macro_deps: Additional proc macro deps to add.
        data: Data to include during compile time
        env: Additional env variables to add when running the script.
        tools: A list of tools needed by the script.
        target_compatible_with: The compatability constraint of the target.
    """

    package_name = native.package_name()

    # Set defaults
    if srcs == None:
        fail("srcs is required")
    if visibility == None:
        visibility = ["//visibility:private"]
    if crate == None:
        crate = package_name.split("/")[-1]
    if deps == None:
        deps = []
    if proc_macro_deps == None:
        proc_macro_deps = []
    if aliases == None:
        aliases = {}
    if data == None:
        data = []
    if features == None:
        features = crate_features(package_name = package_name)
    if target_compatible_with == None:
        target_compatible_with = []

    aliases = aliases | dep_aliases(package_name = package_name, features = features, normal = True, normal_dev = True, proc_macro = True, proc_macro_dev = True)
    all_test_deps = all_crate_deps(normal = True, normal_dev = True, package_name = package_name, features = features) + deps + ["@rules_rust//rust/runfiles", crate]
    all_test_proc_macro_deps = all_crate_deps(proc_macro = True, proc_macro_dev = True, package_name = package_name, features = features) + proc_macro_deps

    rust_binary(
        name = name,
        srcs = srcs,
        crate_features = features.select(),
        aliases = aliases,
        deps = all_test_deps,
        proc_macro_deps = all_test_proc_macro_deps,
        visibility = visibility,
        data = data,
        compile_data = data,
        rustc_flags = [
            "--cfg=bazel_runfiles",
        ] + gc_arg,
        target_compatible_with = target_compatible_with,
    )
