INCLUDE ./sub/a.hurl
[Captures]
greeting: greeting

GET http://localhost:8000/include-dirs/check
X-Greeting: {{greeting}}
HTTP 200
