load("@rules_img//img:load.bzl", "image_load")

DieselMigrationInfo = provider(
    doc = "The result of the diesel migration.",
    fields = {
        "result": "The result of the diesel migration.",
        "schema_file": "The schema file of the diesel migration.",
        "schema_patch_file": "The schema patch file of the diesel migration.",
    },
)

DieselMigrationPatcherInfo = provider(
    doc = "The result of the diesel migration patcher.",
    fields = {
        "result": "The result of the diesel migration patcher.",
    },
)

def _get_rlocation_path(ctx, file):
    """
    Gets the runfiles location path for a file.

    Args:
        ctx: The rule context
        file: A File object

    Returns:
        A string representing the runfiles path
    """
    if file.short_path.startswith("../"):
        # External repository
        return file.short_path[3:]
    else:
        # Main repository
        return ctx.workspace_name + "/" + file.short_path

def _diesel_migration_impl(ctx):
    result = ctx.actions.declare_file(ctx.label.name + ".results.json")
    docker_toolchain = ctx.toolchains["//misc/toolchains/docker:toolchain_type"]

    ctx.actions.run(
        executable = ctx.executable._diesel_migration_tool,
        arguments = [],
        inputs = ctx.files.data + [ctx.file.config_file, ctx.file._rustfmt_config, ctx.file.schema_patch_file],
        outputs = [result],
        env = {
            "DIESEL_CLI_TOOL": ctx.executable._diesel_cli_tool.path,
            "RUSTFMT_TOOL": ctx.file._rustfmt.path,
            "DATABASE_IMAGE_LOAD_TOOL": ctx.executable.database_image_load.path,
            "DIESEL_CONFIG_FILE": ctx.file.config_file.path,
            "OUTPUT_FILE": result.path,
            "RUSTFMT_CONFIG_PATH": ctx.file._rustfmt_config.path,
            "LOADER_BINARY": docker_toolchain.docker_path,
            "SCHEMA_FILE": ctx.file.schema_file.path,
            "SCHEMA_PATCH_FILE": ctx.file.schema_patch_file.path,
        } | docker_toolchain.env,
        tools = [
            ctx.executable._diesel_cli_tool,
            ctx.file._rustfmt,
            ctx.executable.database_image_load,
        ],
    )

    copy_tool = ctx.actions.declare_file(ctx.label.name + "_copy")
    ctx.actions.symlink(
        output = copy_tool,
        target_file = ctx.executable._copy_tool,
        is_executable = True,
    )

    return [
        DefaultInfo(files = depset([result]), executable = copy_tool, runfiles = ctx.runfiles(files = [result])),
        OutputGroupInfo(
            diesel_schema = [result],
        ),
        RunEnvironmentInfo(
            environment = {
                "INPUT_PATH": _get_rlocation_path(ctx, result),
            },
        ),
        DieselMigrationInfo(
            result = result,
            schema_file = ctx.file.schema_file,
            schema_patch_file = ctx.file.schema_patch_file,
        ),
    ]

_diesel_migration = rule(
    implementation = _diesel_migration_impl,
    executable = True,
    attrs = {
        "data": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "schema_file": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "schema_patch_file": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "database_image_load": attr.label(
            mandatory = True,
            executable = True,
            cfg = "exec",
        ),
        "config_file": attr.label(
            mandatory = True,
            allow_single_file = True,
        ),
        "_diesel_cli_tool": attr.label(
            default = "//tools/diesel",
            executable = True,
            cfg = "exec",
        ),
        "_diesel_migration_tool": attr.label(
            default = "//misc/utils/rust/diesel_migration/runner",
            executable = True,
            cfg = "exec",
        ),
        "_rustfmt_config": attr.label(
            default = "//:rustfmt.toml",
            allow_single_file = True,
        ),
        "_rustfmt": attr.label(
            default = "@rules_rust//rust/toolchain:current_rustfmt_toolchain",
            allow_single_file = True,
        ),
        "_copy_tool": attr.label(default = "//misc/utils/rust/diesel_migration/copy", executable = True, cfg = "exec"),
    },
    toolchains = ["//misc/toolchains/docker:toolchain_type"],
)

def _diesel_migration_test_impl(ctx):
    test_binary = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.symlink(output = test_binary, target_file = ctx.executable._diesel_migration_test)

    migration = ctx.attr.diesel_migration[DieselMigrationInfo]
    patcher = ctx.attr.diesel_migration_patcher[DieselMigrationPatcherInfo]

    runfiles = ctx.runfiles(files = [migration.result, migration.schema_file, migration.schema_patch_file, patcher.result])

    return [
        DefaultInfo(executable = test_binary, files = depset([test_binary]), runfiles = runfiles),
        RunEnvironmentInfo(
            environment = {
                "SCHEMA_PATH": _get_rlocation_path(ctx, migration.schema_file),
                "SCHEMA_RESULT_PATH": _get_rlocation_path(ctx, migration.result),
                "SCHEMA_PATCH_PATH": _get_rlocation_path(ctx, migration.schema_patch_file),
                "SCHEMA_PATCH_RESULT_PATH": _get_rlocation_path(ctx, patcher.result),
            },
        ),
    ]

diesel_migration_test = rule(
    implementation = _diesel_migration_test_impl,
    attrs = {
        "diesel_migration": attr.label(
            providers = [DieselMigrationInfo],
            mandatory = True,
        ),
        "diesel_migration_patcher": attr.label(
            providers = [DieselMigrationPatcherInfo],
            mandatory = True,
        ),
        "_diesel_migration_test": attr.label(
            default = "//misc/utils/rust/diesel_migration/test",
            executable = True,
            cfg = "exec",
        ),
    },
    test = True,
)

def _diesel_migration_patcher_impl(ctx):
    output_file = ctx.actions.declare_file(ctx.label.name + "results.json")
    provider = ctx.attr.diesel_migration[DieselMigrationInfo]

    sh_toolchain = ctx.toolchains["@rules_sh//sh/posix:toolchain_type"]

    temp_dir = ctx.actions.declare_directory(ctx.label.name + ".tmp")

    ctx.actions.run(
        executable = ctx.executable._diesel_migration_patcher,
        inputs = [provider.schema_file, provider.schema_patch_file, provider.result],
        outputs = [output_file, temp_dir],
        env = {
            "OUTPUT_FILE": output_file.path,
            "SCHEMA_FILE": provider.schema_file.path,
            "SCHEMA_PATCH_FILE": provider.schema_patch_file.path,
            "SCHEMA_FILE_RESULTS": provider.result.path,
            "TEMP_DIR": temp_dir.path,
            "PATCH_BINARY": sh_toolchain.commands["patch"],
            "DIFF_BINARY": sh_toolchain.commands["diff"],
        },
    )

    copy_tool = ctx.actions.declare_file(ctx.label.name + "_copy")
    ctx.actions.symlink(
        output = copy_tool,
        target_file = ctx.executable._copy_tool,
        is_executable = True,
    )

    runfiles = ctx.runfiles(files = [output_file])

    return [
        DefaultInfo(executable = copy_tool, files = depset([output_file]), runfiles = runfiles),
        RunEnvironmentInfo(
            environment = {
                "INPUT_PATH": _get_rlocation_path(ctx, output_file),
            },
        ),
        DieselMigrationPatcherInfo(
            result = output_file,
        ),
    ]

diesel_migration_patcher = rule(
    implementation = _diesel_migration_patcher_impl,
    executable = True,
    attrs = {
        "diesel_migration": attr.label(
            providers = [DieselMigrationInfo],
            mandatory = True,
        ),
        "_copy_tool": attr.label(default = "//misc/utils/rust/diesel_migration/copy", executable = True, cfg = "exec"),
        "_diesel_migration_patcher": attr.label(
            default = "//misc/utils/rust/diesel_migration/patcher",
            executable = True,
            cfg = "exec",
        ),
    },
    toolchains = ["@rules_sh//sh/posix:toolchain_type"],
)

def diesel_migration(
        name,
        data,
        schema_file,
        schema_patch_file,
        database_image,
        config_file):
    image_load(
        name = "{}_load".format(name),
        image = database_image,
        tag = "bazel-{}:{}".format(native.package_name().replace("/", "-"), name),
    )

    _diesel_migration(
        name = name,
        data = data,
        schema_file = schema_file,
        schema_patch_file = schema_patch_file,
        database_image_load = "{}_load".format(name),
        config_file = config_file,
    )

    diesel_migration_patcher(
        name = "{}_patch".format(name),
        diesel_migration = name,
    )

    diesel_migration_test(
        name = "{}_test".format(name),
        diesel_migration = name,
        diesel_migration_patcher = "{}_patch".format(name),
    )
