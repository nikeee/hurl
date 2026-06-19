GET http://localhost:8000/error-assert-matches-json-schema
HTTP 200
[Asserts]
body matchesJsonSchema "{ \"type\": \"object\", \"required\": [\"name\"] }"
