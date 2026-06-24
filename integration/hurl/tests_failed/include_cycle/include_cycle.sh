#!/bin/bash
set -Eeuo pipefail

hurl tests_failed/include_cycle/include_cycle.hurl
