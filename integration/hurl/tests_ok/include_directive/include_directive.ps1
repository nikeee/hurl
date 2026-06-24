Set-StrictMode -Version latest
$ErrorActionPreference = 'Stop'

hurl --variable username=LEAK tests_ok/include_directive/include_directive.hurl
