INCLUDE ./b.hurl
[Variables]
tag: nested
[Captures]
c_value: final_value

GET http://localhost:8000/include-nested/check
X-Value: {{final_value}}
HTTP 200
