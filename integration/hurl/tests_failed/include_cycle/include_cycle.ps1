Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'

hurl tests_failed/include_cycle/include_cycle.hurl
