# The same single-entry file is included twice: the failure of the first include run must not
# be taken for a retry of the second include run.
INCLUDE ./child.hurl
[Variables]
expected: 500

INCLUDE ./child.hurl
[Variables]
expected: 200
