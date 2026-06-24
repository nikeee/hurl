# Include

## Definition

An `INCLUDE` directive runs another Hurl file as an isolated sub-script. It is the building block for
reusing requests (login flows, shared setup) across test suites without duplicating or generating
Hurl files.

```hurl
INCLUDE ./login.hurl
[Variables]
username: admin
password: {{admin_password}}
[Captures]
token: admin_token
```

An `INCLUDE` is a top-level item, on par with a regular [entry]: a Hurl file is a sequence of entries
and includes, run in order.

## Description

### Path

The path that follows the `INCLUDE` keyword is a [template] (it can contain `{{...}}` placeholders).
It is resolved relative to the directory of the file that contains the `INCLUDE` directive.

```hurl
INCLUDE ./flows/login.hurl
GET https://acmecorp.net/me
HTTP 200
```

### Variables

The optional `[Variables]` section lists the inputs passed to the sub-script, as `name: value` pairs
(values are [templates], evaluated in the *parent* context):

```hurl
INCLUDE ./login.hurl
[Variables]
username: admin
password: {{admin_password}}
```

The sub-script runs in a **fully isolated environment**: its variable set is seeded *only* by the
`[Variables]` section. Command-line variables (`--variable`, `--variables-file`), environment, and
the parent's runtime-captured variables are **not** visible inside the sub-script. This makes it
possible to call the same fragment several times with different inputs without naming collisions.

### Captures

The optional `[Captures]` section promotes variables from the sub-script back into the parent, as a
rename map `sub-script-name: parent-name`:

```hurl
INCLUDE ./login.hurl
[Variables]
username: admin
[Captures]
token: admin_token
```

After the sub-script runs, *only* the variables listed here are copied into the parent. Above, the
sub-script's `token` variable becomes `admin_token` in the parent. Any sub-script variable not listed
is discarded — nothing leaks back implicitly.

If a `[Captures]` entry names a variable the sub-script never defined, the include fails with an
error. A secret captured in the sub-script stays secret when promoted to the parent.

### Isolation

Each included sub-script runs with:

- a fresh variable set, seeded only by `[Variables]` (see above);
- its own HTTP client and cookie store — cookies set during the sub-script are *not* sent on parent
  requests, and vice versa. To carry a session across the boundary, capture the cookie/token into a
  variable and pass it explicitly.

### Reporting

Entries executed inside a sub-script keep their real source location: a failing assertion in an
included file is reported (in the terminal and in the [JUnit report]) against that file and line, not
against the file that contains the `INCLUDE`.

### Reference cycles

Circular includes are detected and rejected. If file A includes B and B includes A (directly or
transitively), Hurl stops with an `Include cycle` error.

### Reserved keyword

Because a line starting with `INCLUDE ` (the keyword followed by a space) is parsed as an include
directive, `INCLUDE` can no longer be used as a custom HTTP method name. Any other method name
(including, for instance, `INCLUDED`) is unaffected.

## Example

```hurl
# user_can_reply_admin.hurl

# Log in as two different users, isolated from each other.
INCLUDE ./login.hurl
[Variables]
username: admin
password: hunter2
[Captures]
cookie: admin_cookie

INCLUDE ./login.hurl
[Variables]
username: user123
password: hunter3
[Captures]
cookie: user_cookie

# Post a comment as the admin, then reply to it as the user.
INCLUDE ./post-comment.hurl
[Variables]
cookie: {{admin_cookie}}
contents: hi
[Captures]
post_id: admin_comment_id

INCLUDE ./post-comment.hurl
[Variables]
cookie: {{user_cookie}}
contents: also hi
parent_id: {{admin_comment_id}}
```

[entry]: https://hurl.dev/docs/entry.html
[template]: https://hurl.dev/docs/templates.html
[templates]: https://hurl.dev/docs/templates.html
[JUnit report]: https://hurl.dev/docs/running-tests.html#generating-report
