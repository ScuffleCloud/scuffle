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
BINARY="$RUNFILES/_main/%%BINARY%%"

# Calculate lib directory relative to binary
LIB_DIR="$(dirname $(dirname "$BINARY"))/lib"

# Set up library path and execute
export LD_LIBRARY_PATH="$LIB_DIR:${LD_LIBRARY_PATH:-}"
exec "$BINARY" "$@"