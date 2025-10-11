#!/usr/bin/env bash

set -euo pipefail

if [[ ${SCUFFLE_RUN_UNDER:-1} == "1" ]]; then
    cd "${BUILD_WORKING_DIRECTORY}"
fi

unset SCUFFLE_RUN_UNDER

exec "${@}"
