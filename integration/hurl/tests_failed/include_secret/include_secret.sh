#!/bin/bash
set -Eeuo pipefail
hurl tests_failed/include_secret/include_secret.hurl
