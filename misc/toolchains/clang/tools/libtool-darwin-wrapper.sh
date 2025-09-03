#!/usr/bin/env bash

set -eu

exec ${LIBTOOL} -static $@
