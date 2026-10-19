#!/bin/bash
set -Eeuo pipefail
hurl --continue-on-error tests_failed/include_continue_on_error/include_continue_on_error.hurl
