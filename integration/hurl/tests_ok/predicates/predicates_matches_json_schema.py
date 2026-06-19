from app import app


@app.route("/predicates-matches-json-schema")
def predicates_matches_json_schema():
    return '{"id": 1, "name": "Bob", "tags": ["a", "b"]}'
