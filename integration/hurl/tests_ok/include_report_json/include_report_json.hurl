INCLUDE ./child.hurl
[Captures]
greeting: child_greeting

GET http://localhost:8000/include-report-json/check
X-Greeting: {{child_greeting}}
HTTP 200
