INCLUDE include_directive_login.hurl
[Variables]
username: admin
[Captures]
token: admin_token

INCLUDE include_directive_login.hurl
[Variables]
username: bob
[Captures]
token: bob_token

GET http://localhost:8000/include-directive/echo?admin={{admin_token}}&bob={{bob_token}}
HTTP 200
[Asserts]
body == "admin=token-admin;bob=token-bob"
