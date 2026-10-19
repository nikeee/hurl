from app import app
from flask import make_response, request


@app.route("/include-isolation/set-cookie")
def include_isolation_set_cookie():
    resp = make_response()
    # The cookie set in an included file must not leak in the including file requests
    # (each include runs with its own cookie jar).
    assert "Cookie" not in request.headers
    resp.set_cookie("isolated", "true")
    return resp


@app.route("/include-isolation/check")
def include_isolation_check():
    assert "Cookie" not in request.headers
    return ""
