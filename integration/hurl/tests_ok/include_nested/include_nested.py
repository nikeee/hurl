from app import app
from flask import jsonify, request


@app.route("/include-nested/value")
def include_nested_value():
    tag = request.args.get("tag")
    return jsonify(value=f"value-{tag}")


@app.route("/include-nested/check")
def include_nested_check():
    assert request.headers["X-Value"] == "value-nested"
    return ""
