GET http://localhost:8000/include-error-child-assert/hello
HTTP 200
[Asserts]
jsonpath "$.greeting" == "goodbye"
