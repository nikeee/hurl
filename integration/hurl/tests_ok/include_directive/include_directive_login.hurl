POST http://localhost:8000/include-directive/login
{
  "username": "{{username}}"
}
HTTP 200
[Captures]
token: jsonpath "$.token"
