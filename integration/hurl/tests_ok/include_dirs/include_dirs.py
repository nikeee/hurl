from app import app
from flask import request


@app.route("/include-dirs/data", methods=["POST"])
def include_dirs_data():
    assert request.data == b"included data\n"
    return ""


@app.route("/include-dirs/check")
def include_dirs_check():
    assert request.headers["X-Greeting"] == "Hello World!"
    return ""
