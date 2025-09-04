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

binary_path="%%BINARY%%"
pwd="$(pwd)"
script_dir=$(dirname_shim "${BASH_SOURCE[0]}")
if [[ ${binary_path} != /* ]]; then
  binary_path="${pwd}/external/${binary_path}"
fi


pwd

args=()
if [[ "$1" == "cq" ]]; then
  shift
  if [[ "$1" == *.a ]]; then
    out="$1"
    shift
    args=(-static -o "$out" "$@")
  else
    echo "wrapper error: expected archive after 'cq'" >&2
    exit 1
  fi
else
  args=("$@")
fi

exec "$binary_path" "${args[@]}"
