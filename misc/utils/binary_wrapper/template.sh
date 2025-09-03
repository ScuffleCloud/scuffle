#!/usr/bin/env bash

set -euo pipefail

dirname_shim() {
  local path="$1"

  # Remove trailing slashes
  path="${path%/}"

  # If there's no slash, return "."
  if [[ "${path}" != */* ]]; then
    echo "."
    return
  fi

  # Remove the last component after the final slash
  path="${path%/*}"

  # If it becomes empty, it means root "/"
  echo "${path:-/}"
}

script="${BASH_SOURCE[0]}"
script_dir=$(dirname_shim "${script}")

# Find the runfiles directory
if [[ -n "${RUNFILES_DIR:-}" ]]; then
    root_dir="${RUNFILES_DIR}"
elif [[ -f "${script_dir}/MANIFEST" ]]; then
    root_dir="${script_dir}"
    export RUNFILES_DIR="${root_dir}"
elif [[ -d "${script}.runfiles" ]]; then
    root_dir="${script}.runfiles"
    export RUNFILES_DIR="${root_dir}"
else
    root_dir="$(pwd)/external/"
fi

if [[ -n "${RUNFILES_DIR:-}" ]]; then
    pwd="${RUNFILES_DIR}/%%WORKSPACE_NAME%%"
else
    pwd="$(pwd)"
fi

# User-specified environment variables
%%EXPORT_ENVS%%

# User-specified extra commands
%%EXTRA_COMMANDS%%

# Set up library path and execute
exec "${root_dir}/%%BINARY%%" "$@"
