#!/bin/bash
set -Eeuo pipefail

hurl tests_failed/assert_matches_json_schema/assert_matches_json_schema.hurl
