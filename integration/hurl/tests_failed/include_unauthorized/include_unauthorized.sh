#!/bin/bash
set -Eeuo pipefail
hurl tests_failed/include_unauthorized/include_unauthorized.hurl
