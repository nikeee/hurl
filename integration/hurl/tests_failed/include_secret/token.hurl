GET http://localhost:8000/include-secret/token
HTTP 200
[Captures]
token: jsonpath "$.token" redact
