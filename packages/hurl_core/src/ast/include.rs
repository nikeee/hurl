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
use super::option::VariableValue;
use super::primitive::{LineTerminator, SourceInfo, Template, Whitespace};

/// Represents an include entry: another Hurl file run as a sub-script, in an isolated
/// variable environment.
///
/// ```hurl
/// INCLUDE ./login.hurl
/// [Variables]
/// username: bob
/// password: {{bob_password}}
/// [Captures]
/// token: bob_token
/// ```
///
/// The included file's variables are seeded from the `[Variables]` section only, and the
/// only values flowing back to the including file are the ones listed in the `[Captures]`
/// section (`child_name: parent_name` rename mapping).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeEntry {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub space1: Whitespace,
    pub path: Template,
    pub line_terminator0: LineTerminator,
    pub sections: Vec<IncludeSection>,
    pub source_info: SourceInfo,
}

impl IncludeEntry {
    /// Returns the variables passed to the included file.
    pub fn variables(&self) -> &[IncludeVariable] {
        for section in &self.sections {
            if let IncludeSectionValue::Variables(variables) = &section.value {
                return variables;
            }
        }
        &[]
    }

    /// Returns the captures exported from the included file to the including file.
    pub fn captures(&self) -> &[IncludeCapture] {
        for section in &self.sections {
            if let IncludeSectionValue::Captures(captures) = &section.value {
                return captures;
            }
        }
        &[]
    }
}

/// Represents a section of an include entry, either `[Variables]` or `[Captures]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeSection {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub line_terminator0: LineTerminator,
    pub value: IncludeSectionValue,
    pub source_info: SourceInfo,
}

impl IncludeSection {
    /// Returns the Hurl string identifier of this section.
    pub fn identifier(&self) -> &'static str {
        match self.value {
            IncludeSectionValue::Variables(_) => "Variables",
            IncludeSectionValue::Captures(_) => "Captures",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncludeSectionValue {
    Variables(Vec<IncludeVariable>),
    Captures(Vec<IncludeCapture>),
}

/// Represents a variable passed to an included file: `name: value`.
///
/// The value is evaluated in the including file's variable environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeVariable {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub name: String,
    pub space1: Whitespace,
    pub space2: Whitespace,
    pub value: VariableValue,
    pub line_terminator0: LineTerminator,
    pub source_info: SourceInfo,
}

/// Represents a variable exported from an included file: `name: alias`.
///
/// `name` is the variable name in the included file, `alias` the variable name it is
/// assigned to in the including file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeCapture {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub name: String,
    pub space1: Whitespace,
    pub space2: Whitespace,
    pub alias: String,
    pub line_terminator0: LineTerminator,
    pub source_info: SourceInfo,
}
