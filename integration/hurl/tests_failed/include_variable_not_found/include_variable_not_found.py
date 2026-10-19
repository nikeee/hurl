from app import app


@app.route("/include-variable-not-found/hello")
def include_variable_not_found_hello():
    return ""
