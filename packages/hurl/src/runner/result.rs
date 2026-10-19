/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2026 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use std::cmp::min;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use hurl_core::ast::SourceInfo;
use hurl_core::input::Input;
use hurl_core::reader::Pos;
use hurl_core::types::Index;

use crate::http::{Call, CookieStore, CurlCmd};
use crate::pretty;
use crate::pretty::PrettyMode;
use crate::pretty::json::Color;
use crate::util::term::Stdout;

use super::error::{RunnerError, RunnerErrorKind};
use super::output::Output;
use super::value::Value;
use super::variable::VariableSet;

/// Represents the result of a valid Hurl file execution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HurlResult {
    /// The entries result for this run.
    pub entries: Vec<EntryResult>,
    /// Total duration of the run, including asserts and results computation.
    pub duration: Duration,
    /// `true` if the run is successful, `false` if there has been runtime or asserts errors.
    pub success: bool,
    /// The cookie store containing the list of cookies at the end of the run.
    pub cookie_store: CookieStore,
    /// Start of the run (in "UNIX timestamp").
    pub timestamp: i64,
    /// The set of variables, updated at the end of the run execution.
    pub variables: VariableSet,
}

impl HurlResult {
    /// Returns all the effective errors of this `HurlResult`, with the source information
    /// of the entry where the error happens and the source file of the entry (`None` if the
    /// entry comes from the run file, and not from an included file).
    ///
    /// The errors are only the "effective" ones: those that are due to retry are
    /// ignored.
    pub fn errors(&self) -> Vec<(&RunnerError, SourceInfo, Option<&Arc<EntrySource>>)> {
        let mut errors = vec![];
        let mut next_entries = self.entries.iter().skip(1);
        for entry in self.entries.iter() {
            match next_entries.next() {
                None => {
                    let new_errors = entry
                        .errors
                        .iter()
                        .map(|error| (error, entry.source_info, entry.source.as_ref()));
                    errors.extend(new_errors);
                }
                Some(next) => {
                    if next.entry_index != entry.entry_index || next.source != entry.source {
                        let new_errors = entry
                            .errors
                            .iter()
                            .map(|error| (error, entry.source_info, entry.source.as_ref()));
                        errors.extend(new_errors);
                    }
                }
            }
        }
        errors
    }
}

/// Represents the source file of an entry, when the entry does not come from the main file
/// of the run but from an included file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntrySource {
    /// The included file.
    pub filename: Input,
    /// The content of the included file.
    pub content: String,
}

/// Represents the execution result of an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryResult {
    /// Index of the entry on the file execution.
    pub entry_index: Index,
    /// Source information of this entry.
    pub source_info: SourceInfo,
    /// List of HTTP request / response pairs. It holds all the requests and responses
    /// that has been executed when following redirections. If `--follow` is not used, this list
    /// contains exactly one call.
    pub calls: Vec<Call>,
    /// List of captures.
    pub captures: Vec<CaptureResult>,
    /// List of asserts.
    pub asserts: Vec<AssertResult>,
    /// List of errors.
    pub errors: Vec<RunnerError>,
    /// Effective duration of all the HTTP transfers, excluding asserts and captures processing.
    pub transfer_duration: Duration,
    /// The entry has been executed with `--compressed` option:
    /// server is requested to send compressed response, and the response should be uncompressed
    /// when outputted on stdout.
    pub compressed: bool,
    /// The debug curl command line from this entry result.
    pub curl_cmd: CurlCmd,
    /// The source file of this entry, when the entry comes from an included file (`None` if
    /// the entry comes from the file of the run). Used to report errors against the included
    /// file content.
    pub source: Option<Arc<EntrySource>>,
}

impl EntryResult {
    /// Returns the source content and filename against which errors of this entry should be
    /// rendered: the entry's own source file if it comes from an included file, or the given
    /// default content and filename (the file of the run) otherwise.
    pub fn source_content<'a>(
        &'a self,
        default_content: &'a str,
        default_filename: Option<&'a Input>,
    ) -> (&'a str, Option<&'a Input>) {
        match &self.source {
            Some(source) => (&source.content, Some(&source.filename)),
            None => (default_content, default_filename),
        }
    }
}

impl Default for EntryResult {
    fn default() -> Self {
        EntryResult {
            entry_index: Index::new(1),
            source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
            calls: vec![],
            captures: vec![],
            asserts: vec![],
            errors: vec![],
            transfer_duration: Duration::from_millis(0),
            compressed: false,
            curl_cmd: CurlCmd::default(),
            source: None,
        }
    }
}

/// Result of a Hurl assertions, either implicit or explicit.
///
/// ## Example
///
/// ```hurl
///  GET https://foo.com
///  HTTP 200
///  x-baz: bar
///  [Asserts]
///  header "toto" == "tutu"
///  jsonpath "$.state" = "running"
/// ```
///
/// In this Hurl sample, everything after the keyword `HTTP` is an assertion. We distinguish two
/// types of assertions: implicit and explicit.
///
/// - `HTTP 200`: implicit status code assert
/// - `x-baz: bar`: implicit HTTP header assert
/// - `header "toto" == "tutu"`: explicit HTTP header assert
/// - `jsonpath "$.state" = "running"`: explicit JSONPath assert on HTTP body response
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssertResult {
    /// Implicit HTTP version assert (like HTTP/3, HTTP/2 etc...).
    ImplicitVersion {
        actual: String,
        expected: String,
        source_info: SourceInfo,
    },
    /// Implicit HTTP status code assert.
    ImplicitStatus {
        actual: u64,
        expected: u64,
        source_info: SourceInfo,
    },
    /// Implicit HTTP response header assert.
    ImplicitHeader {
        actual: Result<String, RunnerError>,
        expected: String,
        source_info: SourceInfo,
    },
    /// Implicit HTTP response body assert.
    ImplicitBody {
        actual: Result<Value, RunnerError>,
        expected: Result<Value, RunnerError>,
        source_info: SourceInfo,
    },
    /// Explicit assert on HTTP response.
    Explicit {
        actual: Result<Option<Value>, RunnerError>,
        source_info: SourceInfo,
        predicate_result: Option<Result<(), RunnerError>>,
    },
}

/// Represents a [capture](https://hurl.dev/docs/capturing-response.html) of an HTTP response.
///
/// Captures are data extracted by querying the HTTP response. Captures can be part of the response
/// body, headers, cookies etc... Captures can be used to re-inject data in next HTTP requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureResult {
    /// Name of the capture.
    pub name: String,
    /// Value of the capture.
    pub value: Value,
}

impl EntryResult {
    /// Writes the last HTTP response of this entry result to this `output`.
    /// The HTTP response can be decompressed if the entry's `compressed` option has been set.
    /// This method doesn't check if the response has write access to this output using a context
    /// directory. Write check access has to be made by the caller of this method
    /// TODO: enforce this by a proper type on output?
    #[allow(clippy::too_many_arguments)]
    pub fn write_response(
        &self,
        output: Option<&Output>,
        stdout: &mut Stdout,
        include_headers: bool,
        color: bool,
        pretty: PrettyMode,
        append: bool,
        source_info: SourceInfo,
    ) -> Result<(), RunnerError> {
        let Some(call) = self.calls.last() else {
            return Ok(());
        };

        let response = &call.response;
        let mut out = Vec::new();

        if include_headers {
            let text = response.get_status_line_headers(color);
            out.append(&mut text.into_bytes());
            out.push(b'\n');
        }

        let body = if self.compressed {
            &response
                .uncompress_body()
                .map_err(|e| RunnerError::new(source_info, RunnerErrorKind::Http(e), false))?
        } else {
            &response.body
        };

        // Prettify only JSON-like response for the moment.
        let pretty = match pretty {
            PrettyMode::Automatic => response.is_json(),
            PrettyMode::Force => true,
            PrettyMode::None => false,
        };
        if pretty {
            let color_pretty = if color { Color::Ansi } else { Color::NoColor };
            // On pretty-print error, discard partial output and fall back to raw bytes.
            let before_len = out.len();
            if pretty::format(body, color_pretty, &mut out).is_err() {
                out.truncate(before_len);
                out.extend_from_slice(body);
            }
        } else {
            out.extend_from_slice(body);
        }

        // We replicate curl's checks for binary output: a warning is displayed when user hasn't
        // used `--output` option and the response is considered as a binary content. If user has used
        // `--output` whether to save to a file, or to redirect output to standard output (`--output -`)
        // we don't display any warning.
        if output.is_none() && stdout.is_terminal() && is_binary(&out) {
            return Err(RunnerError::new(
                source_info,
                RunnerErrorKind::BinaryOutput,
                false,
            ));
        }

        let output = output.unwrap_or(&Output::Stdout);
        output.write(&out, stdout, append).map_err(|e| {
            let kind = RunnerErrorKind::FileWriteAccess {
                path: PathBuf::from(output.to_string()),
                error: e.to_string(),
            };
            RunnerError::new(source_info, kind, false)
        })?;
        Ok(())
    }

    /// Returns `true` if the last HTTP response body ends with a trailing newline.
    pub fn has_response_trailing_newline(&self) -> bool {
        match self.calls.last() {
            None => false,
            Some(call) => call.response.has_trailing_newline(),
        }
    }
}

/// Returns `true` if `bytes` is a binary content, false otherwise.
///
/// For the implementation, we use a simple heuristic on the buffer: just check the presence of NULL
/// in the first 2000 bytes to determine if the content if binary or not.
///
/// See <https://github.com/curl/curl/pull/1512>
/// and <https://github.com/curl/curl/blob/721941aadf4adf4f6aeb3f4c0ab489bb89610c36/src/tool_cb_wrt.c#L209>
fn is_binary(bytes: &[u8]) -> bool {
    let len = min(2000, bytes.len());
    bytes[..len].contains(&0)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hurl_core::ast::SourceInfo;
    use hurl_core::input::Input;
    use hurl_core::reader::Pos;
    use hurl_core::types::Index;

    use super::{EntryResult, EntrySource, HurlResult};
    use crate::runner::error::{RunnerError, RunnerErrorKind};

    fn error_at(line: usize) -> RunnerError {
        RunnerError::new(
            SourceInfo::new(Pos::new(line, 1), Pos::new(line, 1)),
            RunnerErrorKind::NoQueryResult,
            true,
        )
    }

    fn entry(
        index: usize,
        source: Option<Arc<EntrySource>>,
        errors: Vec<RunnerError>,
    ) -> EntryResult {
        EntryResult {
            entry_index: Index::new(index),
            source,
            errors,
            ..Default::default()
        }
    }

    #[test]
    fn errors_dedup_retries_by_index_and_source() {
        let included = Arc::new(EntrySource {
            filename: Input::new("child.hurl"),
            content: "GET http://foo".to_string(),
        });

        // Entry 1 is retried (same index, same source): only the last error is effective.
        // The included entry has the same index as the parent entry, but a different source:
        // its error must not be considered as a retry of the parent entry.
        let entries = vec![
            entry(1, None, vec![error_at(1)]),
            entry(1, None, vec![error_at(1)]),
            entry(1, Some(included.clone()), vec![error_at(2)]),
            entry(2, None, vec![]),
        ];
        let result = HurlResult {
            entries,
            ..Default::default()
        };
        let errors = result.errors();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].2.is_none());
        assert_eq!(errors[1].2, Some(&included));
    }

    #[test]
    fn source_content_selects_included_file() {
        let included = Arc::new(EntrySource {
            filename: Input::new("child.hurl"),
            content: "child content".to_string(),
        });
        let filename = Input::new("parent.hurl");

        let e = entry(1, None, vec![]);
        assert_eq!(
            e.source_content("parent content", Some(&filename)),
            ("parent content", Some(&filename))
        );

        let e = entry(1, Some(included.clone()), vec![]);
        let (content, input) = e.source_content("parent content", Some(&filename));
        assert_eq!(content, "child content");
        assert_eq!(input.unwrap().to_string(), "child.hurl");
    }
}
