load("@rules_rust//cargo:defs.bzl", "cargo_build_script", "cargo_toml_env_vars")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_proc_macro", "rust_test")
load("//vendor:defs.bzl", "all_crate_deps", "crate_features", dep_aliases = "aliases")
load("@rules_shell//shell:sh_test.bzl", "sh_test")

def scuffle_package(
        crate_name,
        features = None,
        crate_type = "rlib",
        srcs = None,
        visibility = None,
        aliases = None,
        deps = None,
        test_deps = None,
        proc_macro_deps = None,
        proc_macro_test_deps = None,
        crate_test = None,
        data = None,
        test_data = None,
        test_env = None,
        test_insta = False,
        tags = None,
        test_tags = None,
    ):
    """Creates a rust_library and corresponding rust_test target.

    Args:
        crate_name: Name of the crate.
        features: A set of features this crate has
        crate_type: The type of crate to build: Default "rlib"
        srcs: Source files. Defaults to glob(["src/**/*.rs"]) if not provided
        visibility: Visibility for the library target. Defaults to ["//visibility:public"]
        aliases: Dependency aliases
        deps: Additional deps to add.
        test_deps: Additional deps to add to the test.
        proc_macro_deps: Additional proc macro deps to add.
        proc_macro_test_deps: Additional proc macro deps to add to tests.
        crate_test: Make a crate test
        data: Data to include during compile time
        test_data: Data to include during runtime
        test_env: Additional env variables when running a test.
        test_insta: Setup the test to work with Insta
        tags: Additional tags to add to the package
        test_tags: Additional tags to add to the test.
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
    if test_deps == None:
        test_deps = []
    if proc_macro_test_deps == None:
        proc_macro_test_deps = []
    if crate_test == None:
        crate_test = crate_type == "rlib"
    if data == None:
        data = []
    if test_data == None:
        test_data = []
    if features == None:
        features = crate_features(package_name = package_name, all = True)
    if test_env == None:
        test_env = {}
    if tags == None:
        tags = []
    if test_tags == None:
        test_tags = []

    NAME_MAPPINGS = {
        "rlib": "lib",
        "bin": "bin",
        "proc_macro": "macro",
    }

    if crate_type not in NAME_MAPPINGS:
        fail("crate_type must be one of: %s" % [kind for kind in NAME_MAPPINGS.keys()])

    name = package_name.split("/")[-1]

    cargo_toml_env_vars(
        name = "cargo_toml_env",
        src = ":Cargo.toml",
        workspace = "//:Cargo.toml",
        visibility = ["//visibility:private"],
    )

    kwargs = dict(
        name = name,
        crate_name = crate_name.replace("-", "_"),
        srcs = srcs,
        crate_features = features.select(),
        aliases = aliases | dep_aliases(package_name = package_name, features = features),
        deps = all_crate_deps(normal = True, package_name = package_name, features = features) + deps,
        proc_macro_deps = all_crate_deps(proc_macro = True, package_name = package_name, features = features) + proc_macro_deps,
        visibility = visibility,
        compile_data = data,
        tags = tags,
        rustc_env_files = [":cargo_toml_env"],
    )

    # Create the library target
    if crate_type == "rlib":
        rust_library(**kwargs)
    elif crate_type == "proc_macro":
        rust_proc_macro(**kwargs)
    elif crate_type == "bin":
        rust_binary(**kwargs)

    if crate_type == "proc_macro":
        proc_macro_test_deps += [":" + name]
    else:
        test_deps += [":" + name]

    if test_insta:
        test_data += native.glob(["src/**"])
        test_tags += ["no-sandbox"]

    if crate_test:
        rust_test(
            name = "__rust_test",
            crate = ":" + name,
            aliases = aliases | dep_aliases(package_name = package_name, features = features),
            deps = all_crate_deps(normal = True, normal_dev = True, package_name = package_name, features = features) + deps + test_deps + ["@rules_rust//rust/runfiles"],
            proc_macro_deps = all_crate_deps(proc_macro = True, proc_macro_dev = True, package_name = package_name, features = features) + proc_macro_deps + proc_macro_test_deps,
            crate_features = features.select(),
            compile_data = data,
            data = test_data,
            env = test_env,
            tags = ["manual"] + test_tags,
            rustc_flags = [
                "--cfg=bazel_test",
            ],
            rustc_env_files = [":cargo_toml_env"],
            visibility = ["//visibility:private"],
        )

        sh_test(
            name = "test",
            srcs = ["//dev-tools/test-runner:script"],
            args = ["$(location //dev-tools/test-runner)", "$(location //:cargo_metadata)", crate_name, "$(location :__rust_test)"],
            data = [":__rust_test", "//dev-tools/test-runner", "//:cargo_metadata"] + test_data,
            tags = test_tags,
            env = test_env,
        )

def scuffle_build_script(
        name,
        features = None,
        srcs = None,
        visibility = None,
        aliases = None,
        deps = None,
        proc_macro_deps = None,
        data = None,
        env = None):
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
    )
