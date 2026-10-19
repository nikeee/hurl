#!/bin/bash
set -Eeuo pipefail

echo "INCLUDE ./tests_ok/include_stdin/child.hurl" | hurl
