GET http://localhost:8000/include-isolation/set-cookie
HTTP 200
[Asserts]
cookie "isolated" == "true"
