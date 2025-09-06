#!/usr/bin/env bash

set -euo pipefail

SCRIPT="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(dirname "${SCRIPT}")"

if [[ -n ${RUNFILES_DIR:-}   ]]; then
    RUNFILES="${RUNFILES_DIR}"
elif [[ -f "${SCRIPT_DIR}/MANIFEST" ]]; then
    RUNFILES="${SCRIPT_DIR}"
else
    RUNFILES="${SCRIPT}.runfiles"
fi

export RUNFILES_DIR="${RUNFILES_DIR}"

# Resolve binary path within runfiles
PROCESS_WRAPPER="$RUNFILES/_main/%%PROCESS_WRAPPER%%"
TARGET_BINARY="$RUNFILES/_main/%%TARGET_BINARY%%"

exec "${PROCESS_WRAPPER}" --subst 'pwd=${pwd}' -- "${TARGET_BINARY}" "$@"
