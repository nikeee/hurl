GET http://localhost:8000/include-nested/value?tag={{tag}}
HTTP 200
[Captures]
value: jsonpath "$.value"
