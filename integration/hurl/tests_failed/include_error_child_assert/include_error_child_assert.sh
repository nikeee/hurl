#!/bin/bash
set -Eeuo pipefail
hurl tests_failed/include_error_child_assert/include_error_child_assert.hurl
