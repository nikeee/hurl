INCLUDE ./token.hurl
[Captures]
token: secret_token

GET http://localhost:8000/include-secret/check
HTTP 200
[Asserts]
variable "secret_token" == "wrong-value"
