from app import app
from flask import jsonify, request


@app.route("/include-directive/login")
def include_directive_login():
    username = request.args.get("username")
    password = request.args.get("password")
    return jsonify(token=f"token-{username}-{password}")


@app.route("/include-directive/check")
def include_directive_check():
    assert request.headers["X-Alice-Token"] == "token-alice-wonderland"
    assert request.headers["X-Bob-Token"] == "token-bob-builder"
    return ""
