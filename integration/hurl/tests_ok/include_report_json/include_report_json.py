from app import app
from flask import request


@app.route("/include-report-json/hello")
def include_report_json_hello():
    return "hello from child"


@app.route("/include-report-json/check")
def include_report_json_check():
    assert request.headers["X-Greeting"] == "hello from child"
    return ""
