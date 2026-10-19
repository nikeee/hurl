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
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use hurl_core::ast::IncludeEntry;
use hurl_core::error::{DisplaySourceError, OutputFormat};
use hurl_core::input::{Input, InputKind};
use hurl_core::parser;
use hurl_core::types::Index;

use crate::util::logger::Logger;
use crate::util::path::ContextDir;
use crate::util::term::Stdout;

use super::error::{RunnerError, RunnerErrorKind};
use super::hurl_file::run_entries_rec;
use super::options::eval_variable_value;
use super::result::{EntryResult, EntrySource};
use super::runner_options::RunnerOptions;
use super::template::eval_template;
use super::value::Value;
use super::variable::VariableSet;

/// Runs the Hurl file included by `include`, in an isolated environment, and returns the results
/// of the included file entries, flattened, with a success flag.
///
/// The included file is run hermetically:
///
/// - its variables are seeded only from the `[Variables]` section of `include` (values being
///   evaluated in the including file's environment `variables`),
/// - it uses a new HTTP client, with a fresh cookie jar,
/// - after a successful run, only the variables listed in the `[Captures]` section of `include`
///   are exported back to the including file's `variables` (`child_name: parent_name` renaming).
///
/// Secret values are propagated in both directions (they are always redacted, on both sides).
///
/// The include path is resolved relatively to the including file `filename` directory, must stay
/// inside the file root of the run (see `--file-root`) and is checked against `visited`, the
/// stack of files currently being run, to detect inclusion cycles.
#[allow(clippy::too_many_arguments)]
pub fn run_include(
    include: &IncludeEntry,
    entry_index: Index,
    filename: Option<&Input>,
    runner_options: &RunnerOptions,
    variables: &mut VariableSet,
    visited: &mut Vec<PathBuf>,
    stdout: &mut Stdout,
    logger: &mut Logger,
) -> (Vec<EntryResult>, bool) {
    let source_info = include.space0.source_info;
    let error_result = |error: RunnerError| {
        let result = EntryResult {
            entry_index,
            source_info,
            errors: vec![error],
            ..Default::default()
        };
        (vec![result], false)
    };

    // Evaluates the include path in the including file's environment.
    let path = match eval_template(&include.path, variables) {
        Ok(path) => PathBuf::from(path),
        Err(error) => return error_result(error),
    };

    // Resolves the path relatively to the including file's directory (or to the current
    // working directory when the including file is standard input).
    let base_dir = match filename.map(Input::kind) {
        Some(InputKind::File(file)) => file.parent().map(Path::to_path_buf).unwrap_or_default(),
        Some(InputKind::Stdin(_)) | None => PathBuf::new(),
    };
    // "." components are removed from the resolved path, they change nothing but clutter error
    // messages and reports (`dir/./a.hurl` instead of `dir/a.hurl`).
    let resolved = base_dir
        .join(&path)
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect::<PathBuf>();

    // The included file must stay inside the file root of the run.
    let context_dir = &runner_options.context_dir;
    if !context_dir.is_access_allowed_resolved(&resolved) {
        let kind = RunnerErrorKind::UnauthorizedFileAccess { path: resolved };
        let error = RunnerError::new(include.path.source_info, kind, false);
        return error_result(error);
    }

    // Checks inclusion cycles on the canonical path: the file must exist to be canonicalized
    // (and to be run).
    let absolute = context_dir.current_dir().join(&resolved);
    let canonical = match fs::canonicalize(&absolute) {
        Ok(canonical) => canonical,
        Err(_) => {
            let kind = RunnerErrorKind::FileReadAccess {
                path: resolved.clone(),
            };
            let error = RunnerError::new(include.path.source_info, kind, false);
            return error_result(error);
        }
    };
    if visited.contains(&canonical) {
        let kind = RunnerErrorKind::IncludeCycle { path: resolved };
        let error = RunnerError::new(include.path.source_info, kind, false);
        return error_result(error);
    }

    // Reads and parses the included file.
    let child_input = Input::from(resolved.clone());
    let child_content = match child_input.read_to_string() {
        Ok(content) => content,
        Err(_) => {
            let kind = RunnerErrorKind::FileReadAccess { path: resolved };
            let error = RunnerError::new(include.path.source_info, kind, false);
            return error_result(error);
        }
    };
    let child_file = match parser::parse_hurl_file(&child_content) {
        Ok(file) => file,
        Err(error) => {
            // The parse error is rendered against the included file content, then a runner
            // error is reported on the include line.
            let message = error.render(
                &child_input.to_string(),
                &child_content,
                None,
                OutputFormat::Terminal(logger.color),
            );
            logger.error_rich(&message);
            let kind = RunnerErrorKind::IncludeParseError { path: resolved };
            let error = RunnerError::new(include.path.source_info, kind, false);
            return error_result(error);
        }
    };

    // Seeds the included file variables: the included file runs in an isolated environment, it
    // only sees the variables of the `[Variables]` section, evaluated in the including file's
    // environment. Parent secret values are propagated so they stay redacted in the included
    // file logs.
    let mut child_variables = VariableSet::new();
    child_variables.extend_secret_values(variables.secrets());
    for variable in include.variables() {
        let value = match eval_variable_value(&variable.value, variables) {
            Ok(value) => value,
            Err(error) => return error_result(error),
        };
        match &value {
            Value::String(s) if variables.secrets().contains(s) => {
                child_variables.insert_secret(variable.name.clone(), s.clone());
            }
            _ => child_variables.insert(variable.name.clone(), value),
        }
    }

    // The included file gets its own context directory: its file accesses and nested includes
    // are relative to its own directory (which is inside the file root of the run).
    let mut child_options = runner_options.clone();
    child_options.from_entry = None;
    child_options.to_entry = None;
    let child_dir = resolved.parent().map(Path::to_path_buf).unwrap_or_default();
    child_options.context_dir = ContextDir::new(context_dir.current_dir(), &child_dir);

    logger.debug_important(&format!("Including file {child_input}"));
    logger.set_secrets(child_variables.secrets());

    // Runs the included file: a new HTTP client (with a fresh cookie jar) is created by
    // `run_entries_rec`.
    visited.push(canonical);
    let child_result = run_entries_rec(
        &child_file.entries,
        &child_content,
        Some(&child_input),
        &child_options,
        &child_variables,
        visited,
        stdout,
        None,
        logger,
    );
    visited.pop();

    // Tags the entry results with the included file source, so errors and reports point to the
    // included file. Entries of deeper includes are already tagged.
    let source = Arc::new(EntrySource {
        filename: child_input,
        content: child_content,
    });
    let mut results = child_result
        .entries
        .into_iter()
        .map(|mut e| {
            if e.source.is_none() {
                e.source = Some(source.clone());
            }
            e
        })
        .collect::<Vec<_>>();

    // Secret values created in the included file are merged back unconditionally (even on
    // failure): a secret remains a secret forever, on both sides.
    variables.extend_secret_values(child_result.variables.secrets());
    logger.set_secrets(variables.secrets());

    if !child_result.success {
        return (results, false);
    }

    // Exports the variables listed in the `[Captures]` section (and only them) to the including
    // file's environment, renaming `name` to `alias` and preserving secretness.
    for capture in include.captures() {
        match child_result.variables.get(&capture.name) {
            None => {
                let kind = RunnerErrorKind::IncludeVariableNotFound {
                    name: capture.name.clone(),
                };
                let error = RunnerError::new(capture.source_info, kind, false);
                let result = EntryResult {
                    entry_index,
                    source_info,
                    errors: vec![error],
                    ..Default::default()
                };
                results.push(result);
                return (results, false);
            }
            Some(variable) => match variable.value() {
                Value::String(value) if variable.is_secret() => {
                    variables.insert_secret(capture.alias.clone(), value.clone());
                }
                value => variables.insert(capture.alias.clone(), value.clone()),
            },
        }
    }
    logger.set_secrets(variables.secrets());

    (results, true)
}
