#!/usr/bin/env bash

_bazel__get_workspace_path() {
  local workspace=$PWD
  while true; do
    if [ -f "${workspace}/WORKSPACE" ] || \
       [ -f "${workspace}/WORKSPACE.bazel" ] || \
       [ -f "${workspace}/MODULE.bazel" ] || \
       [ -f "${workspace}/REPO.bazel" ]; then
      break
    elif [ -z "$workspace" ] || [ "$workspace" = "/" ]; then
      workspace=$PWD
      break;
    fi
    workspace=${workspace%/*}
  done
  echo "$workspace"
}

root_dir="$(_bazel__get_workspace_path)"
bazel_env_bin_dir="${root_dir}/target-bazel/out/bazel_env-opt-ST-6691f78f59fa/bin/bazel_env/bin"

if [[ ! -d "${bazel_env_bin_dir}" ]]; then
    echo "bazel_env_bin_dir does not exist: ${bazel_env_bin_dir}"
    exit 1
fi

export PATH="${bazel_env_bin_dir}:${PATH}"
export WORKSPACE_ROOT="${root_dir}"
export BAZEL_ENV_BIN_DIR="${bazel_env_bin_dir}"
export BAZEL="${WORKSPACE_ROOT}/tools/scripts/bazel"
