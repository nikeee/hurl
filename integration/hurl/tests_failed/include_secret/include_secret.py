from app import app
from flask import jsonify


@app.route("/include-secret/token")
def include_secret_token():
    return jsonify(token="super-secret-token")


@app.route("/include-secret/check")
def include_secret_check():
    return ""
