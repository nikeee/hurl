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
use std::fs;
use std::path::Path;

use crate::report::ReportError;
use crate::runner::{EntryResult, EntrySource, HurlResult, RunnerError};
use hurl_core::ast::SourceInfo;
use hurl_core::input::Input;
use hurl_core::parser;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Testcase {
    /// Unique identifier of this testcase.
    pub id: String,
    /// Source file name.
    pub filename: String,
    pub success: bool,
    pub time_in_ms: u128,
    /// The runtime errors: the error, the source information of the entry throwing this error, and
    /// the source (file and content) of that entry when it comes from an included sub-script.
    pub errors: Vec<(RunnerError, SourceInfo, Option<EntrySource>)>,
    pub timestamp: i64,
}

/// A source document rendered as its own HTML page: the top-level file, or an included sub-script
/// file that contains at least one error.
pub(crate) struct SourceDoc {
    /// Displayed file name.
    pub filename: String,
    /// File content (syntax colored in the page).
    pub content: String,
    /// HTML page file name this document is rendered to.
    pub page: String,
}

impl Testcase {
    /// Creates an HTML testcase.
    pub fn from(hurl_result: &HurlResult, filename: &Input) -> Testcase {
        let id = Uuid::new_v4();
        let errors = hurl_result
            .errors_with_source()
            .into_iter()
            .map(|(error, entry_src_info, source)| (error.clone(), entry_src_info, source.cloned()))
            .collect();
        Testcase {
            id: id.to_string(),
            filename: filename.to_string(),
            time_in_ms: hurl_result.duration.as_millis(),
            success: hurl_result.success,
            errors,
            timestamp: hurl_result.timestamp,
        }
    }

    /// Returns the source documents involved in this testcase: the top-level file first (with
    /// `top_content`), then each distinct included sub-script file that has an error. Each document
    /// is rendered to its own HTML source page.
    pub(crate) fn source_docs(&self, top_content: &str) -> Vec<SourceDoc> {
        let mut docs = vec![SourceDoc {
            filename: self.filename.clone(),
            content: top_content.to_string(),
            page: self.source_filename(),
        }];
        for (_, _, source) in &self.errors {
            if let Some(source) = source {
                let filename = source.filename.to_string();
                if !docs.iter().any(|d| d.filename == filename) {
                    let page = format!("{}-source-{}.html", self.id, docs.len());
                    docs.push(SourceDoc {
                        filename,
                        content: source.content.clone(),
                        page,
                    });
                }
            }
        }
        docs
    }

    /// Returns the source document a given error `source` belongs to: the matching sub-script file,
    /// or the top-level file (the first document) when the error is not from an include.
    pub(crate) fn doc_for<'a>(
        source: &Option<EntrySource>,
        docs: &'a [SourceDoc],
    ) -> &'a SourceDoc {
        match source {
            Some(source) => {
                let filename = source.filename.to_string();
                docs.iter()
                    .find(|d| d.filename == filename)
                    .unwrap_or(&docs[0])
            }
            None => &docs[0],
        }
    }

    /// Exports a [`Testcase`] to HTML in the directory `dir`.
    ///
    /// It will create three HTML files:
    /// - an HTML view of the Hurl source file (with potential errors and syntax colored),
    /// - an HTML timeline view of the executed entries (with potential errors, waterfall)
    /// - an HTML view of the executed run (headers, cookies, etc...)
    ///
    /// `secrets` strings are redacted from the produced HTML.
    pub fn write_html(
        &self,
        content: &str,
        entries: &[EntryResult],
        dir: &Path,
        secrets: &[&str],
    ) -> Result<(), ReportError> {
        // We parse the content as we'll reuse the AST to construct the HTML source file, and
        // the waterfall.
        // TODO: for the moment, we can only have parseable file.
        let hurl_file = parser::parse_hurl_file(content).unwrap();

        // We create the timeline view.
        let output_file = dir.join(self.timeline_filename());
        let html = self.get_timeline_html(&hurl_file, content, entries, secrets);
        fs::write(&output_file, html.as_bytes()).map_err(|e| {
            ReportError::from_io_error(&e, &output_file, "Issue writing HTML report")
        })?;

        // Then create the run view.
        let output_file = dir.join(self.run_filename());
        let html = self.get_run_html(&hurl_file, content, entries, secrets);
        fs::write(&output_file, html.as_bytes()).map_err(|e| {
            ReportError::from_io_error(&e, &output_file, "Issue writing HTML report")
        })?;

        // And create a source view per involved file: the top-level file, plus each included
        // sub-script file that has an error (so errors point at the real sub-file and line).
        let docs = self.source_docs(content);
        for doc in &docs {
            // The content was already parsed when the file ran, so parsing can't fail here.
            let doc_hurl_file = parser::parse_hurl_file(&doc.content).unwrap();
            let output_file = dir.join(&doc.page);
            let html = self.get_source_html(&doc_hurl_file, doc, &docs, content, secrets);
            fs::write(&output_file, html.as_bytes()).map_err(|e| {
                ReportError::from_io_error(&e, &output_file, "Issue writing HTML report")
            })?;
        }

        Ok(())
    }

    pub fn source_filename(&self) -> String {
        format!("{}-source.html", self.id)
    }

    pub fn timeline_filename(&self) -> String {
        format!("{}-timeline.html", self.id)
    }

    pub fn run_filename(&self) -> String {
        format!("{}-run.html", self.id)
    }
}
