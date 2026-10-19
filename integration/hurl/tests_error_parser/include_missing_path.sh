#!/bin/bash
set -Eeuo pipefail
hurl tests_error_parser/include_missing_path.hurl
