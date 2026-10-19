INCLUDE ./login.hurl
[Variables]
username: alice
password: wonderland
[Captures]
token: alice_token

INCLUDE ./login.hurl
[Variables]
username: bob
password: builder
[Captures]
token: bob_token

GET http://localhost:8000/include-directive/check
X-Alice-Token: {{alice_token}}
X-Bob-Token: {{bob_token}}
HTTP 200
