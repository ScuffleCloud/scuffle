#!/usr/bin/env bash

set -euo pipefail

SCRIPT="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(dirname "${SCRIPT}")"

# Find the runfiles directory
if [[ -n "${RUNFILES_DIR:-}" ]]; then
    RUNFILES_DIR="${RUNFILES_DIR}"
elif [[ -f "${SCRIPT_DIR}/MANIFEST" ]]; then
    RUNFILES_DIR="${SCRIPT_DIR}"
else
    RUNFILES_DIR="${SCRIPT}.runfiles"
fi

export RUNFILES_DIR="${RUNFILES_DIR}"

pwd="${RUNFILES_DIR}/_main"

# User-specified environment variables
%%EXPORT_ENVS%%

# User-specified extra commands
%%EXTRA_COMMANDS%%

# Set up library path and execute
exec "${RUNFILES_DIR}/_main/%%BINARY%%" "$@"
