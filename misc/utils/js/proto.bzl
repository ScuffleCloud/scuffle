load("@aspect_bazel_lib//lib:output_files.bzl", "output_files")
load("@aspect_rules_js//npm:defs.bzl", "npm_package")
load("@bazel_skylib//rules:copy_file.bzl", "copy_file")
load("@npm//misc/utils/js:@swc/cli/package_json.bzl", swc_cli_bin = "bin")
load("@npm//misc/utils/js:typescript/package_json.bzl", typescript_bin = "bin")
load("//misc/utils:binary_wrapper.bzl", "binary_wrapper")
load("//misc/utils/protobuf/ts:defs.bzl", "protobuf_ts_compile")

def ts_proto(name, protos):
    """
    Generate the typescript protobuf code.

    Args:
        name: The name of the package.
        protos: The list of protos to generate from, these should be label strings to proto_library targets.
    """

    protobuf_ts_compile(
        name = name + "_proto",
        protos = protos,
    )

    copy_file(
        name = name + "_tsconfig",
        src = "//misc/utils/js/proto:tsconfig.json",
        out = name + "_tsconfig.json",
        allow_symlink = True,
    )

    # Compile that using tsc to get ESM + Types
    typescript_bin.tsc(
        name = name + "_types",
        srcs = [
            ":node_modules",
            ":package.json",
            ":" + name + "_proto",
            ":" + name + "_tsconfig",
        ],
        args = [
            "-p",
            name + "_tsconfig.json",
            "--emitDeclarationOnly",
            "--outDir",
            "dist/types",
        ],
        chdir = native.package_name(),
        out_dirs = ["dist/types"],
    )

    copy_file(
        name = name + "_swcrc",
        src = "//misc/utils/js/proto:.swcrc",
        out = name + "_swcrc.json",
        allow_symlink = True,
    )

    swc_cli_bin.swc(
        name = name + "_esm",
        srcs = [
            ":" + name + "_swcrc",
            ":node_modules",
            ":package.json",
            ":" + name + "_proto",
        ],
        args = [
            "--config-file",
            name + "_swcrc.json",
            "./" + name + "_proto",
            "-d",
            "./dist/esm",
            "--strip-leading-paths",
        ],
        chdir = native.package_name(),
        out_dirs = ["dist/esm"],
    )

    # Since we dont know the outputs ahead of time
    # We are using directory output mode, so we need to select
    # that `generated` directory.
    # This directory is the same one referenced in the `package.json` exports section.
    output_files(
        name = name + "_types_output",
        paths = [native.package_name() + "/dist/types"],
        target = ":" + name + "_types",
    )

    output_files(
        name = name + "_esm_output",
        paths = [native.package_name() + "/dist/esm"],
        target = ":" + name + "_esm",
    )

    # This allows it to be referenced by other projects such as `cloud/dashboard`
    # Unfortunately the name `pkg` is special. So it must be named that.
    npm_package(
        name = name,
        srcs = [
            ":package.json",
            ":" + name + "_esm_output",
            ":" + name + "_types_output",
        ],
        visibility = ["//visibility:public"],
    )

    binary_wrapper(
        name = name + "_dev",
        binary = "//misc/utils/js/proto:dev.sh",
        env = {
            "PACKAGE_NAME": native.package_name(),
            "PKG_PATH": "$(rlocationpath :{name})".format(name = name),
        },
        data = [
            ":" + name,
        ],
    )
