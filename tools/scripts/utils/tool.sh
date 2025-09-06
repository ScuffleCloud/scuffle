#!/usr/bin/env bash

set -euo pipefail

# Get the tool name from the script name (basename of $0)
readonly TOOL_NAME="${TOOL_NAME:-$(basename "$0")}"
declare -A bazel_info_map
while IFS=": " read -r key value; do
    bazel_info_map["$key"]="$value"
done < <(bazel info output_base workspace execution_root 2> /dev/null)

readonly OUTPUT_BASE="${bazel_info_map['output_base']}"
readonly WORKSPACE="${bazel_info_map['workspace']}"
readonly EXECUTION_ROOT="${bazel_info_map['execution_root']}"

readonly CACHE_DIR="${OUTPUT_BASE}/.scripts"
readonly CACHE_PATH="${CACHE_DIR}/${TOOL_NAME}.cache"

# Configuration mapping tool names to their bazel labels
declare -Ar TOOL_LABELS=(
     ["cargo"]="//tools/cargo"
     ["cargo-deny"]="//tools/cargo/deny"
     ["cargo-insta"]="//tools/cargo/insta"
     ["buf"]="//tools/buf"
     ["buildifier"]="//tools/buildifier"
     ["dprint"]="//tools/dprint"
     ["ffmpeg"]="//tools/ffmpeg"
     ["ffprobe"]="//tools/ffprobe"
     ["protoc"]="//tools/protoc"
     ["node"]="//tools/node"
     ["pnpm"]="//tools/pnpm"
     ["just"]="//tools/just"
     ["miniserve"]="//tools/miniserve"
     ["valgrind"]="//tools/valgrind"
     ["ibazel"]="//tools/ibazel"
     ["bazel-diff"]="//tools/bazel-diff"
     ["rust-analyzer"]="//tools/rust-analyzer"
     ["rust-analyzer-discover"]="//tools/rust-analyzer:discover"
     ["rust-analyzer-check"]="//tools/rust-analyzer:check"
     ["shfmt"]="//tools/shfmt"
)

declare -Ar pnpm_vars=(
     ["SCUFFLE_RUN_UNDER"]="0"
)

declare -a pnpm_args=(
    "--dir=$(pwd)"
)

# Error handling function
die() {
    printf "$*\n" >&2
    exit 1
}

is_cache_valid() {
    [[ -f ${CACHE_PATH}   ]] || return 1

    local first_line
    read -r first_line < "$CACHE_PATH" || return 1
    local ts="${first_line%%=*}"
    local exec_path="${first_line#*=}"

    local max_age="${TOOL_CACHE_MAX_AGE:-86400}"
    local now="$(date +%s)"
    ((now - ts < max_age)) || return 1

    [[ -n ${exec_path} && -f "${EXECUTION_ROOT}/${exec_path}"   ]]
}

# Get tool configuration
get_tool_config() {
    local var_name="${TOOL_NAME//-/_}"

    if declare -p "${var_name}_vars" &> /dev/null; then
        local -n env_vars="${var_name}_vars"
        for key in "${!env_vars[@]}"; do
            export "$key=${env_vars[$key]}"
        done
    fi

    if declare -p "${var_name}_args" &> /dev/null; then
        local -n args_array="${var_name}_args"
        local -n dest_array=$1
        dest_array=("${args_array[@]}")
    fi
}

# Build the tool and get its executable path + env
build_tool() {
    local tool_label="$1"
    local tool_args="$2"

    local stderr_output
    stderr_output="$(mktemp)" || die "Failed to create temporary file for stderr"
    trap "rm -f '${stderr_output}'" EXIT

    local result
    if ! result="$(bazel cquery "$tool_label" \
        --build \
        --output=starlark \
        --color=yes \
        --starlark:expr="'\n'.join(
            ['$(date +%s)=' + providers(target)['FilesToRunProvider'].executable.path] +
            ['%s=%s' % (k, v) for k,v in sorted(
                (providers(target)['RunEnvironmentInfo'].environment if 'RunEnvironmentInfo' in providers(target) else {}).items()
            )]
        )" \
        $tool_args 2> "${stderr_output}")"; then
        die "bazel cquery failed:\n$(cat "${stderr_output}")"
    fi

    echo "$result" >> "$CACHE_PATH"

    rm -f "${stderr_output}"
}

# Execute the tool
invoke_bazel_exec() {
    local first_line
    read -r first_line < "$CACHE_PATH" || return 1
    local exec_path="${first_line#*=}"

    local pwd="$(pwd)"
    local runfiles_dir="${EXECUTION_ROOT}/${exec_path}.runfiles/_main"

    if [[ -d ${runfiles_dir}   ]]; then
        cd "${runfiles_dir}" || die "Failed to change to runfiles directory"
    fi

    # Load env vars from cache file (skip first line)
    set -a
    source <(tail -n +2 "$CACHE_PATH")
    set +a

    exec env \
        -u JAVA_RUNFILES \
        -u RUNFILES_DIR \
        -u RUNFILES_MANIFEST_FILE \
        -u RUNFILES_MANIFEST_ONLY \
        -u TEST_SRCDIR \
        BUILD_WORKING_DIRECTORY="${pwd}" \
        BUILD_WORKSPACE_DIRECTORY="${WORKSPACE}" \
        "${WORKSPACE}/misc/utils/run_under.sh" "${EXECUTION_ROOT}/${exec_path}" "$@"
}

main() {
    [[ -n ${TOOL_NAME}   ]] || die "Tool name cannot be empty"

    local tool_label="${TOOL_LABELS[${TOOL_NAME}]:-}"
    [[ -n ${tool_label}   ]] || {
        echo "Unknown tool '${TOOL_NAME}'" >&2
        echo "Available tools: ${!TOOL_LABELS[*]}" >&2
        exit 1
    }

    mkdir -p "${CACHE_DIR}" || die "Failed to create cache directory: ${CACHE_DIR}"

    local tool_args="${TOOL_ARGS:-}"

    local extra_args=()
    get_tool_config extra_args

    if ! is_cache_valid; then
        build_tool "$tool_label" "$tool_args"
    fi

    invoke_bazel_exec "${extra_args[@]}" "$@"
}

main "$@"
