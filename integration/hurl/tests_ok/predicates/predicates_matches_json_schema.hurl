GET http://localhost:8000/predicates-matches-json-schema
HTTP 200
[Asserts]
# Inline (quoted) JSON schema:
body matchesJsonSchema "{ \"type\": \"object\", \"required\": [\"name\"] }"
# Multiline string JSON schema:
body matchesJsonSchema ```
{
  "type": "object",
  "properties": {
    "id": { "type": "integer" },
    "name": { "type": "string" }
  },
  "required": ["id", "name"]
}
```
# JSON schema loaded from a file:
body matchesJsonSchema file,predicates_matches_json_schema.schema.json;
# The predicate also works on a structured query result:
jsonpath "$.tags" matchesJsonSchema "{ \"type\": \"array\" }"
# The body is not an array:
body not matchesJsonSchema "{ \"type\": \"array\" }"
