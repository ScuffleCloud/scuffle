#!/usr/bin/env bash

_bazel__get_workspace_path() {
    local workspace=$PWD
    while true; do
        if [ -f "${workspace}/WORKSPACE" ] \
            || [ -f "${workspace}/WORKSPACE.bazel" ] \
            || [ -f "${workspace}/MODULE.bazel" ] \
            || [ -f "${workspace}/REPO.bazel" ]; then
            break
        elif [ -z "$workspace" ] || [ "$workspace" = "/" ]; then
            workspace=$PWD
            break
        fi
        workspace=${workspace%/*}
    done
    echo "$workspace"
}

root_dir="$(_bazel__get_workspace_path)"
case "$(uname -s)" in
    Linux)
        platform_name="linux"
        ;;
    Darwin)
        platform_name="macos"
        ;;
    *)
        log_error "ERROR[.envrc]: Unsupported platform: $(uname -s)"
        ;;
esac

bazel_env_bin_dir="target-bazel/out/bazel_env-opt-ST-6691f78f59fa/bin/tools/bazel_env_${platform_name}/bin"
if [[ ! -d ${bazel_env_bin_dir} ]]; then
    echo "bazel_env_bin_dir does not exist: ${bazel_env_bin_dir}"
    exit 1
fi

export PATH="${bazel_env_bin_dir}:${PATH}"
export WORKSPACE_ROOT="${root_dir}"
export BAZEL_ENV_BIN_DIR="${bazel_env_bin_dir}"
export BAZEL="${WORKSPACE_ROOT}/tools/scripts/bazel"
