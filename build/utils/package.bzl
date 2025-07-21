load("@rules_rust//cargo:defs.bzl", "cargo_build_script", "cargo_toml_env_vars", "extract_cargo_lints")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_proc_macro")
load("@rules_rust//rust:defs.bzl", "rustfmt_test")
load("//build/utils:clippy.bzl", "rust_clippy", "rust_clippy_test")
load("//build/utils:rustdoc.bzl", "rustdoc", "rustdoc_test")
load("//build/utils:nextest_test.bzl", "nextest_test")
load("@cargo_vendor//:defs.bzl", "all_crate_deps", "crate_features", dep_aliases = "aliases", "crate_version")

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
    extra_target_kwargs = None,
):
    """Creates a rust_library and corresponding rust_test target.

    Args:
        crate_name: Name of the crate.
        name: Name of the target
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
        visibility = ["//visibility:private"],
    )

    extract_cargo_lints(
        name = name + "_cargo_toml_lints",
        manifest = ":Cargo.toml",
        workspace = "//:Cargo.toml",
        tags = ["manual"],
        visibility = ["//visibility:private"],
    )

    colon_name = ":" + name

    normal_deps = all_crate_deps(normal = True, package_name = package_name, features = features) + deps + ["@rules_rust//rust/runfiles"]
    normal_proc_macro_deps = all_crate_deps(proc_macro = True, package_name = package_name, features = features) + proc_macro_deps
    aliases = aliases | dep_aliases(package_name = package_name, features = features)

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
        rustc_flags = [
            "--cfg=bazel_runfiles",
            "-Clink-arg=-Wl,-znostart-stop-gc",
        ],
        rustc_env_files = [colon_name + "_cargo_toml_env"],
    )

    # Create the library target
    if crate_type == "rlib":
        rust_library(**kwargs)
    elif crate_type == "proc_macro":
        rust_proc_macro(**kwargs)
    elif crate_type == "bin":
        rust_binary(**kwargs)

    rust_targets = [colon_name]

    if test != False:
        test_deps = test.get("deps", [])[:]
        test_proc_macro_deps = test.get("proc_macro_deps", []) + []
        test_env = test.get("env", {}) | {}
        test_data = test.get("data", []) + []
        test_insta = test.get("insta", False)
        test_tags = test.get("tags", []) + []

        if crate_type == "proc_macro":
            test_proc_macro_deps += [colon_name]
        else:
            test_deps += [colon_name]

        workspace_root = None
        if test_insta:
            test_data += native.glob(["src/**/*"])
            workspace_root = "//:test_workspace_root"

        all_test_deps = all_crate_deps(normal = True, normal_dev = True, package_name = package_name, features = features) + deps + test_deps + ["@rules_rust//rust/runfiles"]
        all_test_proc_macro_deps = all_crate_deps(proc_macro = True, proc_macro_dev = True, package_name = package_name, features = features) + proc_macro_deps + test_proc_macro_deps

        nextest_test(
            name = name + "_test",
            workspace_root = workspace_root,
            data = test_data,
            env = test_env,
            tags = test_tags,
            crate = colon_name,
            aliases = aliases | dep_aliases(package_name = package_name, features = features),
            deps = all_test_deps,
            proc_macro_deps = all_test_proc_macro_deps,
            crate_features = features.select(),
            rustc_env_files = [colon_name + "_cargo_toml_env"],
            rustc_flags = [
                "--cfg=bazel_runfiles",
                "--cfg=coverage_nightly",
                "-Clink-arg=-Wl,-znostart-stop-gc",
            ],
            rustc_env = {
                "RUSTC_BOOTSTRAP": "1",
            },
            # Needs to be marked as not testonly because the rust_clippy
            # rule depends on this, which we use to generate clippy suggestions
            testonly = False,
            visibility = ["//visibility:private"],
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
            rustc_flags = [
                "--cfg=bazel_runfiles",
                "-Clink-arg=-Wl,-znostart-stop-gc",
            ],
        )

        rust_targets.append(colon_name + "_test")

    rust_clippy(
        name = name + "_clippy",
        targets = rust_targets,
        visibility = ["//visibility:private"],
    )

    rust_clippy_test(
        name = name + "_clippy_test",
        targets = [colon_name + "_clippy"],
        visibility = ["//visibility:private"],
    )

    rustfmt_test(
        name = name + "_fmt_test",
        targets = rust_targets,
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
    )

def scuffle_test(
    deps = None,
    proc_macro_deps = None,
    env = None,
    data = None,
    insta = False,
    tags = None,
):
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
    return {
        "deps": deps,
        "proc_macro_deps": proc_macro_deps,
        "env": env,
        "data": data,
        "insta": insta,
        "tags": tags,
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
        tools = None):
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

    cargo_build_script(
        name = name,
        srcs = srcs,
        crate_features = features.select(),
        aliases = aliases | dep_aliases(package_name = package_name, features = features, build = True, build_proc_macro = True),
        deps = all_crate_deps(package_name = package_name, features = features, build = True) + deps,
        proc_macro_deps = all_crate_deps(package_name = package_name, features = features, build_proc_macro = True) + proc_macro_deps,
        visibility = visibility,
        data = data,
        compile_data = data,
        build_script_env = env,
        tools = tools,
        rustc_flags = [
            "--cfg=bazel_runfiles",
            "-Clink-arg=-Wl,-znostart-stop-gc",
        ],
    )
