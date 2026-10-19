GET http://localhost:8000/include-directive/login?username={{username}}&password={{password}}
HTTP 200
[Captures]
token: jsonpath "$.token"
