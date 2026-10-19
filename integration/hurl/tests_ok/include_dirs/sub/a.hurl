# Included files are resolved relatively to the including file, and can go up as long as they
# stay under the file root of the run (here, the directory of the run file).
INCLUDE ../common.hurl
[Captures]
greeting: greeting

# File bodies are resolved against the file root of the run, like in any Hurl file.
POST http://localhost:8000/include-dirs/data
file,data.txt;
HTTP 200
