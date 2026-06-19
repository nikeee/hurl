from app import app


@app.route("/error-assert-matches-json-schema")
def error_assert_matches_json_schema():
    return '{"id": 1}'
