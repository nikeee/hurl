#!/bin/bash
set -Eeuo pipefail

rm -rf build/report/include_json

hurl --test --report-json build/report/include_json tests_ok/include_report_json/include_report_json.hurl

cat build/report/include_json/report.json
