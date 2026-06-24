from app import app
from flask import request, make_response


@app.route("/include-directive/login", methods=["POST"])
def include_directive_login():
    username = request.json["username"]
    resp = make_response('{"token": "token-' + username + '"}')
    resp.headers["Content-Type"] = "application/json"
    return resp


@app.route("/include-directive/echo")
def include_directive_echo():
    admin = request.args.get("admin")
    bob = request.args.get("bob")
    return make_response(f"admin={admin};bob={bob}")
