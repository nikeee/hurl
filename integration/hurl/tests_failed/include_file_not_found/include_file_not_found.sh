#!/bin/bash
set -Eeuo pipefail
hurl tests_failed/include_file_not_found/include_file_not_found.hurl
