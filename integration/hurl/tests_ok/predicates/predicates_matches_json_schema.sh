#!/bin/bash
set -Eeuo pipefail

hurl tests_ok/predicates/predicates_matches_json_schema.hurl
