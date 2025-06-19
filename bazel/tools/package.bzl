load("//vendor:defs.bzl", "all_crate_deps", "aliases", "crate_features")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_proc_macro", "rust_test", "rust_binary")

def scuffle_package(
    crate_name,
    features = None,
    crate_type = "rlib",
    srcs = None,
    visibility = None,
    deps = None,
    proc_macro_deps = None,
    crate_test = None,
    data = None,
    test_data = None
):
    """Creates a rust_library and corresponding rust_test target.

    Args:
        crate_name: Name of the crate.
        features: A set of features this crate has
        crate_type: The type of crate to build: Default "rlib"
        srcs: Source files. Defaults to glob(["src/**/*.rs"]) if not provided
        visibility: Visibility for the library target. Defaults to ["//visibility:public"]
        deps: Additional deps to add.
        proc_macro_deps: Additional proc macro deps to add.
        crate_test: Make a crate test
        data: Data to include during compile time
        test_data: Data to include during runtime
    """

    package_name = native.package_name()

    # Set defaults
    if srcs == None:
        srcs = native.glob(["src/**/*.rs"])
    if visibility == None:
        visibility = ["//visibility:public"]
    if deps == None:
        deps = []
    if proc_macro_deps == None:
        proc_macro_deps = []
    if crate_test == None:
        crate_test = crate_type == "rlib"
    if data == None:
        data = []
    if test_data == None:
        test_data = []
    if features == None:
        features = crate_features(package_name = package_name)

    NAME_MAPPINGS = {
        "rlib": "lib",
        "bin": "bin",
        "proc_macro": "macro",
    }

    if crate_type not in NAME_MAPPINGS:
        fail("crate_type must be one of: %s" % [kind for kind in NAME_MAPPINGS.keys()])

    name = package_name.split('/')[-1]

    kwargs = dict(
        name = name,
        crate_name = crate_name.replace("-", "_"),
        srcs = srcs,
        crate_features = features.select(),
        aliases = aliases(package_name = package_name, features = features),
        deps = all_crate_deps(normal = True, package_name = package_name, features = features) + deps,
        proc_macro_deps = all_crate_deps(proc_macro = True, package_name = package_name, features = features) + proc_macro_deps,
        visibility = visibility,
        compile_data = data,
    )

    # Create the library target
    if crate_type == "rlib":
        rust_library(**kwargs)
    elif crate_type == "proc_macro":
        rust_proc_macro(**kwargs)
    elif crate_type == "bin":
        rust_binary(**kwargs)

    test_deps = all_crate_deps(normal = True, normal_dev = True, package_name = package_name, features = features) + deps
    test_proc_macro_deps = all_crate_deps(proc_macro = True, proc_macro_dev = True, package_name = package_name, features = features) + proc_macro_deps


    if crate_type == "proc_macro":
        test_proc_macro_deps.append(":" + name)
    else:
        test_deps.append(":" +name)

    if crate_test:
        rust_test(
            name = name + "_test",
            crate = ":" +name,
            aliases = aliases(
                normal_dev = True,
                proc_macro_dev = True,
                package_name = package_name,
                features = features,
            ),
            deps = test_deps,
            proc_macro_deps = test_proc_macro_deps,
            compile_data = data,
            data = test_data
        )
