#!/usr/bin/env bash

set -euo pipefail

realpath=$(realpath "${RUNFILES_DIR}/${PKG_PATH}")
output_path="${BUILD_WORKSPACE_DIRECTORY}/${PACKAGE_NAME}/dist"

rm -rf "${output_path}" || true
ln -s "${realpath}/dist" "${output_path}"

echo "setup ${PACKAGE_NAME}/dist"
