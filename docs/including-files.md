# Including Files

## Include

A Hurl file can run another Hurl file as a sub-script with the `INCLUDE` statement. Includes are
useful to reuse sequences of requests (like login flows) across a test suite, without duplicating
them in every file.

```hurl
# Log two different users in, reusing the same login file:
INCLUDE ./login.hurl
[Variables]
username: alice
password: wonderland
[Captures]
token: alice_token

INCLUDE ./login.hurl
[Variables]
username: bob
password: builder
[Captures]
token: bob_token

# Use the two exported tokens in the including file:
GET https://example.org/api/messages
X-Alice-Token: {{alice_token}}
X-Bob-Token: {{bob_token}}
HTTP 200
```

With `login.hurl` being:

```hurl
POST https://example.org/api/login
[Form]
username: {{username}}
password: {{password}}
HTTP 200
[Captures]
token: jsonpath "$.token"
```

## Structure

An include consists of the `INCLUDE` keyword, followed by the path of the file to run, and two
optional sections: a `[Variables]` section, that defines the variables of the included file, and
a `[Captures]` section, that exports variables from the included file run back to the including
file.

The path is resolved relatively to the directory of the file containing the `INCLUDE` statement,
and can use [templates]:

```hurl
INCLUDE ./fragments/{{scenario}}.hurl
```

Like any file access in Hurl, the included file must stay under the file root of the run (by
default the directory of the run file, configurable with [`--file-root`]). File bodies
(`file,data.txt;`) and outputs of an included file are resolved against the file root of the run
too, like in any Hurl file.

## Isolation

The included file runs in an isolated environment:

- its variables are only the ones defined in the `[Variables]` section: variables from the
command line (`--variable`, `--variables-file`, `HURL_` environment variables) and variables or
captures of the including file are not visible in the included file,
- it uses its own HTTP session, with a fresh cookie jar: cookies of the including file are not
sent by the included file requests, and cookies set during the included file run don't leak into
the including file session,
- after the run, only the variables listed in the `[Captures]` section of the include are
exported to the including file.

This makes the included file a function-like unit: what goes in and out is entirely explicit. In
particular, two includes of the same login file don't overwrite each other's session cookies:
session values are passed around explicitly through captures.

Command line options (like [`--retry`], [`--insecure`] or [`--cookie`]) apply to the requests of
the included file as to any request of the run.

### Passing variables in

The `[Variables]` section defines the variables of the included file, with `name: value` pairs.
Values can be booleans, numbers, `null`, strings and [templates], evaluated in the including
file's variable environment:

```hurl
INCLUDE ./post-comment.hurl
[Variables]
cookie: {{user_cookie}}
contents: also hi
count: 3
```

### Exporting captures out

The `[Captures]` section lists the variables exported from the included file run, with
`name: alias` pairs: `name` is the variable name inside the included file (usually created by a
[capture]), `alias` is the variable name assigned in the including file:

```hurl
INCLUDE ./post-comment.hurl
[Captures]
post_id: user_comment_id
```

After this include, the including file can use `{{user_comment_id}}`, holding the value of the
`post_id` variable at the end of the included file run. If the included file run doesn't define
the `name` variable, the include fails with a runtime error.

[Secrets] stay secrets across includes: a [redacted capture] exported to the including file
remains redacted there, and secret values passed to an included file are redacted from the
included file logs and reports.

## Nesting and cycles

An included file can itself include other files (paths being resolved relatively to its own
directory). Inclusion cycles are detected at runtime and fail the run with an error:

```shell
error: Include cycle
  --> tests/a.hurl:1:9
   |
 1 | INCLUDE ./b.hurl
   |         ^^^^^^^^ file tests/b.hurl is already being run, including it creates a cycle
   |
```

Including the same file several times is allowed (each run being isolated), only real cycles are
rejected.

## Errors and reports

Entries of an included file appear as entries of the run: errors, [HTML, JSON, JUnit and TAP
reports] reference the included file name and line numbers, so a failure inside an included file
points to the original included file:

```shell
error: Assert failure
  --> tests/login.hurl:4:0
   |
   | POST https://example.org/api/login
   | ...
 4 | jsonpath "$.token" exists
   |   actual:   none
   |   expected: something
   |
```

By default, Hurl outputs the body of the last HTTP response of the run: when an include is the last
entry of a file, the body written is the one of the last response of the included file.

## Limitations

- An `INCLUDE` statement has no `[Options]` section: options like `retry`, `repeat`, `skip` or
`delay` can't be set on an include (they can be used inside the included file, on its requests).
- For [`--from-entry` and `--to-entry`], an include counts as a single entry of the including
file: the included file always runs entirely.
- `INCLUDE` is a reserved keyword: a custom HTTP method named `INCLUDE` can't be used.

[templates]: /docs/templates.md
[capture]: /docs/capturing-response.md
[Secrets]: /docs/templates.md#secrets
[redacted capture]: /docs/capturing-response.md#redacting-secrets
[HTML, JSON, JUnit and TAP reports]: /docs/running-tests.md#generating-report
[`--file-root`]: /docs/manual.md#file-root
[`--retry`]: /docs/manual.md#retry
[`--insecure`]: /docs/manual.md#insecure
[`--cookie`]: /docs/manual.md#cookie
[`--from-entry` and `--to-entry`]: /docs/manual.md#from-entry
