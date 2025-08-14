#!/usr/bin/env bash

set -euo pipefail

# Get the directory where this script is located (preserving symlinks)
SCRIPT="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(dirname "${SCRIPT}")"

# Find the runfiles directory
if [[ -n "${RUNFILES_DIR:-}" ]]; then
    RUNFILES="${RUNFILES_DIR}"
elif [[ -f "${SCRIPT_DIR}/MANIFEST" ]]; then
    RUNFILES="${SCRIPT_DIR}"
else
    RUNFILES="${SCRIPT}.runfiles"
fi

pwd="${RUNFILES}/_main"

# User-specified environment variables
%%EXPORT_ENVS%%

# Set up library path and execute
exec "$RUNFILES/_main/%%BINARY%%" "$@"
