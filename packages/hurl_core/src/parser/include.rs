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
use crate::ast::{
    IncludeCapture, IncludeEntry, IncludeSection, IncludeSectionValue, IncludeVariable, SourceInfo,
};
use crate::combinator::{non_recover, recover, zero_or_more};
use crate::parser::primitives::{
    line_terminator, literal, one_or_more_spaces, optional_line_terminators, zero_or_more_spaces,
};
use crate::parser::sections::section_name;
use crate::parser::{ParseError, ParseErrorKind, ParseResult, filename, option};
use crate::reader::{Pos, Reader};

/// Parses an include entry.
///
/// ```hurl
/// INCLUDE ./login.hurl
/// [Variables]
/// username: bob
/// [Captures]
/// token: bob_token
/// ```
///
/// If the leading keyword is not `INCLUDE`, a recoverable error is returned and the reader is
/// reset, so the entry can be parsed as a request. Once the `INCLUDE` keyword has been read,
/// every error is non-recoverable.
pub fn include_entry(reader: &mut Reader) -> ParseResult<IncludeEntry> {
    let entry_start = reader.cursor();
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let start = reader.cursor();
    let keyword = reader.read_while(|c| c.is_ascii_alphabetic());
    if keyword != "INCLUDE" {
        reader.seek(entry_start);
        let kind = ParseErrorKind::Expecting {
            value: "INCLUDE".to_string(),
        };
        return Err(ParseError::new(start.pos, true, kind));
    }
    let space1 = non_recover(one_or_more_spaces, reader)?;
    let path = filename::parse(reader)?;
    let line_terminator0 = non_recover(line_terminator, reader)?;
    let sections = include_sections(reader)?;
    let source_info = SourceInfo::new(entry_start.pos, reader.cursor().pos);

    check_duplicated_sections(&sections)?;

    Ok(IncludeEntry {
        line_terminators,
        space0,
        space1,
        path,
        line_terminator0,
        sections,
        source_info,
    })
}

fn include_sections(reader: &mut Reader) -> ParseResult<Vec<IncludeSection>> {
    let sections = zero_or_more(include_section, reader)?;
    Ok(sections)
}

fn include_section(reader: &mut Reader) -> ParseResult<IncludeSection> {
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let start = reader.cursor();
    let name = section_name(reader)?;
    let source_info = SourceInfo::new(start.pos, reader.cursor().pos);

    let line_terminator0 = line_terminator(reader)?;
    let value = match name.as_str() {
        "Variables" => section_value_variables(reader)?,
        "Captures" => section_value_captures(reader)?,
        _ => {
            let kind = ParseErrorKind::IncludeSectionName { name: name.clone() };
            let pos = Pos::new(start.pos.line, start.pos.column + 1);
            return Err(ParseError::new(pos, false, kind));
        }
    };

    Ok(IncludeSection {
        line_terminators,
        space0,
        line_terminator0,
        value,
        source_info,
    })
}

fn check_duplicated_sections(sections: &[IncludeSection]) -> Result<(), ParseError> {
    let mut section_names = vec![];
    for section in sections {
        if section_names.contains(&section.identifier()) {
            return Err(ParseError::new(
                section.source_info.start,
                false,
                ParseErrorKind::DuplicateSection,
            ));
        } else {
            section_names.push(section.identifier());
        }
    }
    Ok(())
}

fn section_value_variables(reader: &mut Reader) -> ParseResult<IncludeSectionValue> {
    let items = zero_or_more(include_variable, reader)?;
    Ok(IncludeSectionValue::Variables(items))
}

fn section_value_captures(reader: &mut Reader) -> ParseResult<IncludeSectionValue> {
    let items = zero_or_more(include_capture, reader)?;
    Ok(IncludeSectionValue::Captures(items))
}

fn include_variable(reader: &mut Reader) -> ParseResult<IncludeVariable> {
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let start = reader.cursor();
    let name = include_variable_name(reader)?;
    let space1 = zero_or_more_spaces(reader)?;
    recover(|p1| literal(":", p1), reader)?;
    let space2 = zero_or_more_spaces(reader)?;
    let value = option::variable_value(reader)?;
    let line_terminator0 = line_terminator(reader)?;
    let source_info = SourceInfo::new(start.pos, reader.cursor().pos);
    Ok(IncludeVariable {
        line_terminators,
        space0,
        name,
        space1,
        space2,
        value,
        line_terminator0,
        source_info,
    })
}

fn include_capture(reader: &mut Reader) -> ParseResult<IncludeCapture> {
    let line_terminators = optional_line_terminators(reader)?;
    let space0 = zero_or_more_spaces(reader)?;
    let start = reader.cursor();
    let name = include_variable_name(reader)?;
    let space1 = zero_or_more_spaces(reader)?;
    recover(|p1| literal(":", p1), reader)?;
    let space2 = zero_or_more_spaces(reader)?;
    let alias = option::variable_name(reader)?;
    let line_terminator0 = line_terminator(reader)?;
    let source_info = SourceInfo::new(start.pos, reader.cursor().pos);
    Ok(IncludeCapture {
        line_terminators,
        space0,
        name,
        space1,
        space2,
        alias,
        line_terminator0,
        source_info,
    })
}

/// Parses a variable name, at the start of a `[Variables]` or `[Captures]` line.
///
/// A missing name is a recoverable error (it ends the section), while an invalid (reserved)
/// name is a hard error.
fn include_variable_name(reader: &mut Reader) -> ParseResult<String> {
    match option::variable_name(reader) {
        Ok(name) => Ok(name),
        Err(error) => {
            let recoverable = matches!(error.kind, ParseErrorKind::Expecting { .. });
            Err(ParseError::new(error.pos, recoverable, error.kind))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::VariableValue;
    use crate::parser::parse_hurl_file;
    use crate::types::ToSource;

    #[test]
    fn test_include_entry() {
        let mut reader = Reader::new("INCLUDE ./login.hurl\n");
        let include = include_entry(&mut reader).unwrap();
        assert_eq!(include.path.to_string(), "./login.hurl");
        assert!(include.sections.is_empty());
        assert!(include.variables().is_empty());
        assert!(include.captures().is_empty());
    }

    #[test]
    fn test_include_entry_with_sections() {
        let content = "INCLUDE ./login.hurl\n\
                       [Variables]\n\
                       username: bob\n\
                       count: 3\n\
                       verbose: true\n\
                       empty: null\n\
                       password: {{bob_password}}\n\
                       [Captures]\n\
                       token: bob_token\n";
        let mut reader = Reader::new(content);
        let include = include_entry(&mut reader).unwrap();
        assert_eq!(include.path.to_string(), "./login.hurl");
        assert_eq!(include.sections.len(), 2);

        let variables = include.variables();
        assert_eq!(variables.len(), 5);
        assert_eq!(variables[0].name, "username");
        assert_eq!(variables[0].value.to_string(), "bob");
        assert!(matches!(variables[1].value, VariableValue::Number(_)));
        assert!(matches!(variables[2].value, VariableValue::Bool(true)));
        assert!(matches!(variables[3].value, VariableValue::Null));
        assert_eq!(variables[4].value.to_string(), "{{bob_password}}");

        let captures = include.captures();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].name, "token");
        assert_eq!(captures[0].alias, "bob_token");
    }

    #[test]
    fn test_include_entry_with_template_path() {
        let mut reader = Reader::new("INCLUDE {{dir}}/login.hurl\n");
        let include = include_entry(&mut reader).unwrap();
        assert_eq!(include.path.to_source().as_str(), "{{dir}}/login.hurl");
    }

    #[test]
    fn test_include_entry_missing_path() {
        let mut reader = Reader::new("INCLUDE \n");
        let error = include_entry(&mut reader).unwrap_err();
        assert!(!error.recoverable);
        assert_eq!(error.kind, ParseErrorKind::Filename);
    }

    #[test]
    fn test_include_entry_invalid_section() {
        let mut reader = Reader::new("INCLUDE ./login.hurl\n[Asserts]\n");
        let error = include_entry(&mut reader).unwrap_err();
        assert!(!error.recoverable);
        assert_eq!(
            error.kind,
            ParseErrorKind::IncludeSectionName {
                name: "Asserts".to_string()
            }
        );
        assert_eq!(error.pos, Pos::new(2, 2));
    }

    #[test]
    fn test_include_entry_duplicate_section() {
        let content = "INCLUDE ./login.hurl\n\
                       [Variables]\n\
                       a: b\n\
                       [Variables]\n\
                       c: d\n";
        let mut reader = Reader::new(content);
        let error = include_entry(&mut reader).unwrap_err();
        assert!(!error.recoverable);
        assert_eq!(error.kind, ParseErrorKind::DuplicateSection);
    }

    #[test]
    fn test_include_entry_reserved_variable_name() {
        let mut reader = Reader::new("INCLUDE ./login.hurl\n[Variables]\nnewUuid: x\n");
        let error = include_entry(&mut reader).unwrap_err();
        assert!(!error.recoverable);
    }

    #[test]
    fn test_include_keyword_mismatch_is_recoverable() {
        // A method that starts like INCLUDE must be parsed as a request.
        let mut reader = Reader::new("INCLUDED http://example.org\n");
        let error = include_entry(&mut reader).unwrap_err();
        assert!(error.recoverable);
        // The reader is reset so the request parser can take over.
        assert_eq!(reader.cursor().pos, Pos::new(1, 1));
    }

    #[test]
    fn test_hurl_file_with_includes() {
        let content = "GET http://example.org/login\n\
                       HTTP 200\n\
                       [Captures]\n\
                       admin_cookie: cookie \"session\"\n\
                       \n\
                       INCLUDE ./post-comment.hurl\n\
                       [Variables]\n\
                       cookie: {{admin_cookie}}\n\
                       [Captures]\n\
                       post_id: admin_comment_id\n\
                       \n\
                       GET http://example.org/comments/{{admin_comment_id}}\n\
                       HTTP 200\n";
        let file = parse_hurl_file(content).unwrap();
        assert_eq!(file.entries.len(), 3);
        assert!(file.entries[0].as_request().is_some());
        let include = file.entries[1].as_include().unwrap();
        assert_eq!(include.path.to_string(), "./post-comment.hurl");
        assert_eq!(include.variables().len(), 1);
        assert_eq!(include.captures().len(), 1);
        assert!(file.entries[2].as_request().is_some());
    }

    #[test]
    fn test_hurl_file_with_trailing_include() {
        let content = "INCLUDE ./a.hurl\n\nINCLUDE ./b.hurl\n\n\n";
        let file = parse_hurl_file(content).unwrap();
        assert_eq!(file.entries.len(), 2);
        assert_eq!(
            file.entries[0].as_include().unwrap().path.to_string(),
            "./a.hurl"
        );
        assert_eq!(
            file.entries[1].as_include().unwrap().path.to_string(),
            "./b.hurl"
        );
    }

    #[test]
    fn test_include_entry_source_info() {
        let mut reader = Reader::new("INCLUDE ./login.hurl\n[Variables]\na: b\n");
        let include = include_entry(&mut reader).unwrap();
        assert_eq!(include.source_info.start, Pos::new(1, 1));
        let variables = include.variables();
        assert_eq!(variables[0].source_info.start, Pos::new(3, 1));
    }
}
