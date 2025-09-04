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

find_binary_path() {
  local binary_path="$1"
  local pwd="$2"
  local script_dir="$3"

  # If already absolute, use as-is
  if [[ ${binary_path} == /* ]]; then
    echo "$binary_path"
    return
  fi

  # Try current directory + external/ (execroot case)
  if [[ -f "${pwd}/external/${binary_path}" ]]; then
    echo "${pwd}/external/${binary_path}"
    return
  fi

  # Try script directory + external/ (runfiles case with external in same dir)
  if [[ -f "${script_dir}/external/${binary_path}" ]]; then
    echo "${script_dir}/external/${binary_path}"
    return
  fi

  # Try going up from script directory to find execroot
  local current_dir="$script_dir"
  while [[ "$current_dir" != "/" ]]; do
    if [[ -d "${current_dir}/external" && -f "${current_dir}/external/${binary_path}" ]]; then
      echo "${current_dir}/external/${binary_path}"
      return
    fi
    current_dir=$(dirname_shim "$current_dir")
  done

  # If we're in runfiles, try to find the workspace root
  # Runfiles typically have a structure like: .../runfiles/workspace_name/...
  if [[ "$script_dir" == */runfiles/* ]]; then
    # Extract the runfiles root
    local runfiles_root="${script_dir%/runfiles/*}/runfiles"

    # Try looking in the runfiles root for external
    if [[ -f "${runfiles_root}/external/${binary_path}" ]]; then
      echo "${runfiles_root}/external/${binary_path}"
      return
    fi

    # Try looking for a workspace directory that contains external
    for workspace_dir in "${runfiles_root}"/*; do
      if [[ -d "$workspace_dir" && -f "${workspace_dir}/external/${binary_path}" ]]; then
        echo "${workspace_dir}/external/${binary_path}"
        return
      fi
    done
  fi

  # Fallback to original behavior
  echo "${pwd}/external/${binary_path}"
}

binary_path="%%BINARY%%"
pwd="$(pwd)"
script_dir=$(dirname_shim "${BASH_SOURCE[0]}")

# Find the correct binary path
resolved_binary_path=$(find_binary_path "$binary_path" "$pwd" "$script_dir")

args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    cq|-cq)
      shift
      if [[ -z "${1:-}" || "$1" != *.a ]]; then
        echo "wrapper error: expected 'libtool cq <archive>.a <objects...>'" >&2
        exit 1
      fi
      args+=(-static -o "$1")
      shift
      ;;
    s|-s)
      # Ignore strip
      shift
      ;;
    *)
      args+=("$1")
      shift
      ;;
  esac
done

exec "$resolved_binary_path" "${args[@]}"
