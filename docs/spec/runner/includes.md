# Includes

This document describes the runtime design of the `INCLUDE` statement (see
[the user documentation](../../including-files.md) for the feature itself and
[the grammar](../grammar/hurl.grammar) for the syntax).

```hurl
INCLUDE ./post-comment.hurl
[Variables]
cookie: {{user_cookie}}
contents: also hi
[Captures]
post_id: user_comment_id
```

## Execution model

An include is an entry of the including file. When the runner main loop (`run_entries_rec` in
`packages/hurl/src/runner/hurl_file.rs`) hits an `Entry::Include`, it delegates to
`runner::include::run_include` (`packages/hurl/src/runner/include.rs`), which:

1. evaluates the path template in the _including_ file's variable environment,
2. resolves the path relatively to the including file's directory (the current working directory
   when the including file is standard input),
3. checks that the resolved path stays under the file root of the run (`ContextDir`, same
   sandbox as `file,...` bodies and `output`),
4. canonicalizes the path and checks it against the stack of files currently being run (cycle
   detection, see below),
5. reads and parses the included file — a parse error is rendered against the included file
   content, then reported as an `IncludeParseError` on the include line,
6. builds the included file's variable set (isolation, see below),
7. recursively calls `run_entries_rec` on the included file's entries, with a child
   `RunnerOptions` (cleared `from_entry`/`to_entry`, `ContextDir` rooted at the included file's
   directory) — a new HTTP client, hence a fresh cookie jar, is created per `run_entries_rec`
   call,
8. flattens the included run's entry results into the including run's results, tagging each of
   them with the included file source (see reporting below),
9. exports the captures listed on the include back to the including file's variables.

## Isolation

The included file's `VariableSet` is built from scratch: only the `[Variables]` pairs of the
include (evaluated in the _including_ file's environment) are inserted. CLI variables, parent
captures and parent locals are not visible in the included file.

The cookie jar is not shared in either direction: each `run_entries_rec` call creates its own
HTTP client. Sessions must be passed around explicitly, through captured values.

On success, for each `name: alias` pair of the include's `[Captures]` section, the value of
`name` in the included run's final variable set is inserted as `alias` in the including file's
variable set. A missing `name` is a runner error (`IncludeVariableNotFound`) located at the
capture line. Nothing else flows back.

## Secrets

Secret _values_ are propagated in both directions, unconditionally (even when the included run
fails), because a value that is secret anywhere must be redacted everywhere:

- the parent's secret values are copied into the child's redaction set before the run (a secret
  templated into a `[Variables]` value stays redacted in the child's logs),
- the child's secret values are merged back into the parent's redaction set after the run, and
  the logger is refreshed.

Variable secretness is preserved on both boundaries: a secret parent variable passed as a
`[Variables]` value yields a secret child variable, and a redacted child capture exported through
`[Captures]` yields a secret parent variable (`VariableSet::insert_secret`).

## Cycle detection

`run_entries` seeds a stack of canonicalized paths (`Vec<PathBuf>`) with the run file, and each
include pushes the included file's canonical path before recursing and pops it after. Including a
file already on the stack is an `IncludeCycle` error at the include line.

Because it's a stack and not a set, including the same file twice _sequentially_ is legal (each
run is isolated); only real cycles are rejected.

## Reporting

`EntryResult` has a `source: Option<Arc<EntrySource>>` field (`EntrySource` = included filename +
content). Entries of the run file have `source: None`; entries of an included file are tagged
with their file (entries of nested includes are tagged once, at the innermost level, and kept
as-is by outer levels).

All error rendering and reports select the content/filename per entry (`EntryResult::
source_content`): stderr messages, HTML report (errors from included files are listed but not
line-linked into the run file source view), JSON report (`entries[].filename` is set for
included entries), JUnit failures. Retry deduplication (`HurlResult::errors`, `is_success`)
compares `(entry_index, source)` since child-local indices can collide with parent indices after
flattening.

The default standard output (the body of the last response of the run, see `output::write_last_body`)
follows the flattened results too: when an include is the last entry of the run file, the body
written is the one of the last response of the included file.

## Interaction with options

An include has no `[Options]` section: `retry`, `repeat`, `delay`, `skip` don't apply to the
include itself. `--from-entry`/`--to-entry` count an include as one entry of the including file
and are not forwarded into the child run (the included file always runs entirely). The `--test`
progress listener reports at the including file's entry granularity.
