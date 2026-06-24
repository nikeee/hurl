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
use hurl_core::ast::HurlFile;

use crate::report::html::Testcase;
use crate::report::html::nav::Tab;
use crate::report::html::testcase::SourceDoc;

impl Testcase {
    /// Returns the HTML string of a Hurl source `doc` (syntax colored and errors).
    ///
    /// `docs` is the list of all source documents of this testcase (used to attribute each error to
    /// its file), and `top_content` is the top-level file content (used by the shared navigation
    /// component to list every error).
    pub(crate) fn get_source_html(
        &self,
        hurl_file: &HurlFile,
        doc: &SourceDoc,
        docs: &[SourceDoc],
        top_content: &str,
        secrets: &[&str],
    ) -> String {
        let nav = self.get_nav_html(top_content, Tab::Source, secrets);
        let nav_css = include_str!("resources/nav.css");
        let source_div = hurl_core::format::format_html(hurl_file, false);
        // Only mark the lines of the errors that belong to this document.
        let error_lines = self
            .errors
            .iter()
            .filter(|(_, _, source)| Testcase::doc_for(source, docs).filename == doc.filename)
            .map(|(error, _, _)| error.source_info.start.line)
            .collect::<Vec<_>>();
        let lines_div = get_numbered_lines(&doc.content, &error_lines);
        let source_css = include_str!("resources/source.css");
        let hurl_css = hurl_core::format::hurl_css();
        format!(
            include_str!("resources/source.html"),
            filename = doc.filename,
            hurl_css = hurl_css,
            lines_div = lines_div,
            nav = nav,
            nav_css = nav_css,
            source_div = source_div,
            source_css = source_css,
        )
    }
}

/// Returns a list of lines number in HTML, marking `error_lines` as errors.
fn get_numbered_lines(content: &str, error_lines: &[usize]) -> String {
    let errors = error_lines;
    let mut lines =
        content
            .lines()
            .enumerate()
            .fold("<pre><code>".to_string(), |acc, (count, _)| -> String {
                let line = count + 1;
                let tag = if errors.contains(&line) {
                    format!("<a id=\"l{line}\" href=\"#l{line}\" class=\"line-error\">{line}</a>\n")
                } else {
                    format!("<a id=\"l{line}\" href=\"#l{line}\">{line}</a>\n")
                };
                acc + &tag
            });
    lines.push_str("</pre></code>");
    lines
}
