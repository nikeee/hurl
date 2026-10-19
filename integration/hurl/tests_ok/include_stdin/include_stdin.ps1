Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'

echo "INCLUDE ./tests_ok/include_stdin/child.hurl" | hurl
