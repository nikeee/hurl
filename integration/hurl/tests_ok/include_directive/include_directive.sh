#!/bin/bash
set -Eeuo pipefail

# `--variable username=LEAK` must NOT leak into the included sub-scripts (input isolation): each
# include uses the values from its own [Variables] section (admin and bob).
hurl --variable username=LEAK tests_ok/include_directive/include_directive.hurl
