from app import app
from flask import jsonify


@app.route("/include-error-child-assert/hello")
def include_error_child_assert_hello():
    return jsonify(greeting="hello")
