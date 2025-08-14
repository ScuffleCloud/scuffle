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
    # Fallback: assume runfiles directory is adjacent to script
    RUNFILES="${SCRIPT}.runfiles"
fi

# Resolve binary path within runfiles
PROCESS_WRAPPER="$RUNFILES/_main/%%PROCESS_WRAPPER%%"
TARGET_BINARY="$RUNFILES/_main/%%TARGET_BINARY%%"

exec "${PROCESS_WRAPPER}" --subst 'pwd=${pwd}' -- "${TARGET_BINARY}" "$@"
