Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'

if (Test-Path build/report/include_json) {
    Remove-Item -Recurse build/report/include_json
}

hurl --test --report-json build/report/include_json tests_ok/include_report_json/include_report_json.hurl

Write-Host (Get-Content build/report/include_json/report.json -Raw) -NoNewLine
