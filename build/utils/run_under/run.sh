#!/usr/bin/env bash

set -euo pipefail

runfiles="${1}.runfiles"
if [[ -d "${runfiles}" ]]; then
    export RUNFILES_DIR="${runfiles}"
    if [[ -f "${runfiles}/MANIFEST" ]]; then
        export RUNFILES_MANIFEST_FILE="${runfiles}/MANIFEST"
    fi
fi

cd "${BUILD_WORKING_DIRECTORY}"

exec "${@}"
