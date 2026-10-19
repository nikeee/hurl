#!/bin/bash
set -Eeuo pipefail
hurl tests_failed/include_cycle_transitive/include_cycle_transitive.hurl
