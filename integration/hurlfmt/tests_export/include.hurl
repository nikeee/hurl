GET http://localhost:8000/login
HTTP 200
[Captures]
admin_cookie: cookie "session"

INCLUDE ./post-comment.hurl
[Variables]
cookie: {{admin_cookie}}
contents: also hi
count: 3
flag: true
nothing: null
[Captures]
post_id: admin_comment_id
