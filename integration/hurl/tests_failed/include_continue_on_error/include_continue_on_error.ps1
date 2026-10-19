Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'
hurl --continue-on-error tests_failed/include_continue_on_error/include_continue_on_error.hurl
