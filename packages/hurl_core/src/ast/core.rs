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
use std::fmt;

use crate::types::{SourceString, ToSource};

use super::option::EntryOption;
use super::primitive::{
    Bytes, I64, KeyValue, LineTerminator, Placeholder, SourceInfo, Template, Whitespace,
};
use super::section::{Assert, Capture, Cookie, MultipartParam, RegexValue, Section, SectionValue};

/// Represents Hurl AST root node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HurlFile {
    pub entries: Vec<EntryKind>,
    pub line_terminators: Vec<LineTerminator>,
}

/// Represents a top-level item of a Hurl file: either a request/response [`Entry`] or an
/// [`Include`] directive that runs another Hurl file as an isolated sub-script.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum EntryKind {
    Request(Entry),
    Include(Include),
}

impl EntryKind {
    /// Returns the source information for this top-level item.
    pub fn source_info(&self) -> SourceInfo {
        match self {
            EntryKind::Request(entry) => entry.source_info(),
            EntryKind::Include(include) => include.source_info,
        }
    }
}

/// Represents an entry; a request AST specification to be run and an optional response AST
/// specification to be checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub request: Request,
    pub response: Option<Response>,
}

impl Entry {
    /// Returns the source information for this entry.
    pub fn source_info(&self) -> SourceInfo {
        self.request.space0.source_info
    }
}

/// Represents an `INCLUDE` directive: it runs another Hurl file as an isolated sub-script.
///
/// The sub-script is executed in a fresh, isolated environment: its variables are seeded only by
/// the (optional) `[Variables]` section, it uses its own HTTP client (cookie store), and only the
/// variables listed in the (optional) `[Captures]` section are copied back into the parent (under
/// their renamed name).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Include {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub space1: Whitespace,
    pub path: Template,
    pub line_terminator0: LineTerminator,
    pub variables: Option<IncludeVariablesSection>,
    pub captures: Option<IncludeCapturesSection>,
    pub source_info: SourceInfo,
}

impl Include {
    /// Returns the variables passed to the sub-script (empty if there is no `[Variables]` section).
    pub fn variables(&self) -> &[KeyValue] {
        self.variables.as_ref().map_or(&[], |s| &s.items)
    }

    /// Returns the captures promoted from the sub-script to the parent (empty if there is no
    /// `[Captures]` section).
    pub fn captures(&self) -> &[IncludeCapture] {
        self.captures.as_ref().map_or(&[], |s| &s.items)
    }
}

/// The `[Variables]` section of an [`Include`]: inputs seeded into the sub-script environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeVariablesSection {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub line_terminator0: LineTerminator,
    pub items: Vec<KeyValue>,
    pub source_info: SourceInfo,
}

/// The `[Captures]` section of an [`Include`]: a rename map promoting sub-script variables back
/// into the parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeCapturesSection {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub line_terminator0: LineTerminator,
    pub items: Vec<IncludeCapture>,
    pub source_info: SourceInfo,
}

/// A single rename entry of an [`Include`] `[Captures]` section: `name: target` where `name` is a
/// variable defined in the sub-script and `target` is the name it takes in the parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeCapture {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub name: Template,
    pub space1: Whitespace,
    pub space2: Whitespace,
    pub target: Template,
    pub line_terminator0: LineTerminator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub method: Method,
    pub space1: Whitespace,
    pub url: Template,
    pub line_terminator0: LineTerminator,
    pub headers: Vec<KeyValue>,
    pub sections: Vec<Section>,
    pub body: Option<Body>,
    pub source_info: SourceInfo,
}

impl Request {
    /// Returns the query strings params for this request.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/API/URL/searchParams>.
    pub fn querystring_params(&self) -> &[KeyValue] {
        for section in &self.sections {
            if let SectionValue::QueryParams(params, _) = &section.value {
                return params;
            }
        }
        &[]
    }

    /// Returns the form params for this request.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/POST#url-encoded_form_submission>.
    pub fn form_params(&self) -> &[KeyValue] {
        for section in &self.sections {
            if let SectionValue::FormParams(params, _) = &section.value {
                return params;
            }
        }
        &[]
    }

    /// Returns the multipart form data for this request.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/POST#multipart_form_submission>.
    pub fn multipart_form_data(&self) -> &[MultipartParam] {
        for section in &self.sections {
            if let SectionValue::MultipartFormData(params, _) = &section.value {
                return params;
            }
        }
        &[]
    }

    /// Returns the list of cookies on this request.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cookie>.
    pub fn cookies(&self) -> &[Cookie] {
        for section in &self.sections {
            if let SectionValue::Cookies(cookies) = &section.value {
                return cookies;
            }
        }
        &[]
    }

    /// Returns the basic authentication on this request.
    ///
    /// See <https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication>.
    pub fn basic_auth(&self) -> Option<&KeyValue> {
        for section in &self.sections {
            if let SectionValue::BasicAuth(kv) = &section.value {
                return kv.as_ref();
            }
        }
        None
    }

    /// Returns the options specific for this request.
    pub fn options(&self) -> &[EntryOption] {
        for section in &self.sections {
            if let SectionValue::Options(options) = &section.value {
                return options;
            }
        }
        &[]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub line_terminators: Vec<LineTerminator>,
    pub version: Version,
    pub space0: Whitespace,
    pub status: Status,
    pub space1: Whitespace,
    pub line_terminator0: LineTerminator,
    pub headers: Vec<KeyValue>,
    pub sections: Vec<Section>,
    pub body: Option<Body>,
    pub source_info: SourceInfo,
}

impl Response {
    /// Returns the captures list of this spec response.
    pub fn captures(&self) -> &[Capture] {
        for section in self.sections.iter() {
            if let SectionValue::Captures(captures) = &section.value {
                return captures;
            }
        }
        &[]
    }

    /// Returns the asserts list of this spec response.
    pub fn asserts(&self) -> &[Assert] {
        for section in self.sections.iter() {
            if let SectionValue::Asserts(asserts) = &section.value {
                return asserts;
            }
        }
        &[]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Method(String);

impl Method {
    /// Creates a new AST element method/
    pub fn new(method: &str) -> Method {
        Method(method.to_string())
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSource for Method {
    fn to_source(&self) -> SourceString {
        self.0.to_source()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub value: VersionValue,
    pub source_info: SourceInfo,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VersionValue {
    Version1,
    Version11,
    Version2,
    Version3,
    VersionAny,
}

impl fmt::Display for VersionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VersionValue::Version1 => "HTTP/1.0",
            VersionValue::Version11 => "HTTP/1.1",
            VersionValue::Version2 => "HTTP/2",
            VersionValue::Version3 => "HTTP/3",
            VersionValue::VersionAny => "HTTP",
        };
        write!(f, "{s}")
    }
}

impl ToSource for VersionValue {
    fn to_source(&self) -> SourceString {
        self.to_string().to_source()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub value: StatusValue,
    pub source_info: SourceInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusValue {
    Any,
    Specific(u64),
}

impl fmt::Display for StatusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusValue::Any => write!(f, "*"),
            StatusValue::Specific(v) => write!(f, "{v}"),
        }
    }
}

impl ToSource for StatusValue {
    fn to_source(&self) -> SourceString {
        self.to_string().to_source()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Body {
    pub line_terminators: Vec<LineTerminator>,
    pub space0: Whitespace,
    pub value: Bytes,
    pub line_terminator0: LineTerminator,
}

/// Check that variable name is not reserved
/// (would conflicts with an existing function)
pub fn is_variable_reserved(name: &str) -> bool {
    ["getEnv", "newDate", "newUuid"].contains(&name)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub source_info: SourceInfo,
    pub value: FilterValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterValue {
    Base64Decode,
    Base64Encode,
    Base64UrlSafeDecode,
    Base64UrlSafeEncode,
    CharsetDecode {
        space0: Whitespace,
        encoding: Template,
    },
    Count,
    DaysAfterNow,
    DaysBeforeNow,
    Decode {
        space0: Whitespace,
        encoding: Template,
    },
    First,
    Format {
        space0: Whitespace,
        fmt: Template,
    },
    DateFormat {
        space0: Whitespace,
        fmt: Template,
    },
    HtmlEscape,
    HtmlUnescape,
    JsonPath {
        space0: Whitespace,
        expr: Template,
    },
    Last,
    Location,
    Nth {
        space0: Whitespace,
        n: IntegerValue,
    },
    Regex {
        space0: Whitespace,
        value: RegexValue,
    },
    Replace {
        space0: Whitespace,
        old_value: Template,
        space1: Whitespace,
        new_value: Template,
    },
    ReplaceRegex {
        space0: Whitespace,
        pattern: RegexValue,
        space1: Whitespace,
        new_value: Template,
    },
    Split {
        space0: Whitespace,
        sep: Template,
    },
    ToDate {
        space0: Whitespace,
        fmt: Template,
    },
    ToFloat,
    ToHex,
    ToInt,
    ToString,
    UrlDecode,
    UrlEncode,
    UrlQueryParam {
        space0: Whitespace,
        param: Template,
    },
    Utf8Decode,
    Utf8Encode,
    XPath {
        space0: Whitespace,
        expr: Template,
    },
}

impl FilterValue {
    /// Returns the Hurl identifier for this filter type.
    pub fn identifier(&self) -> &'static str {
        match self {
            FilterValue::Base64Decode => "base64Decode",
            FilterValue::Base64Encode => "base64Encode",
            FilterValue::Base64UrlSafeDecode => "base64UrlSafeDecode",
            FilterValue::Base64UrlSafeEncode => "base64UrlSafeEncode",
            FilterValue::CharsetDecode { .. } => "charsetDecode",
            FilterValue::Count => "count",
            FilterValue::DaysAfterNow => "daysAfterNow",
            FilterValue::DaysBeforeNow => "daysBeforeNow",
            FilterValue::Decode { .. } => "decode",
            FilterValue::First => "first",
            FilterValue::Format { .. } => "format",
            FilterValue::DateFormat { .. } => "dateFormat",
            FilterValue::HtmlEscape => "htmlEscape",
            FilterValue::HtmlUnescape => "htmlUnescape",
            FilterValue::JsonPath { .. } => "jsonpath",
            FilterValue::Last => "last",
            FilterValue::Location => "location",
            FilterValue::Nth { .. } => "nth",
            FilterValue::Regex { .. } => "regex",
            FilterValue::Replace { .. } => "replace",
            FilterValue::ReplaceRegex { .. } => "replaceRegex",
            FilterValue::Split { .. } => "split",
            FilterValue::ToDate { .. } => "toDate",
            FilterValue::ToFloat => "toFloat",
            FilterValue::ToHex => "toHex",
            FilterValue::ToInt => "toInt",
            FilterValue::ToString => "toString",
            FilterValue::UrlDecode => "urlDecode",
            FilterValue::UrlEncode => "urlEncode",
            FilterValue::UrlQueryParam { .. } => "urlQueryParam",
            FilterValue::Utf8Decode => "utf8Decode",
            FilterValue::Utf8Encode => "utf8Encode",
            FilterValue::XPath { .. } => "xpath",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntegerValue {
    Literal(I64),
    Placeholder(Placeholder),
}

impl fmt::Display for IntegerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegerValue::Literal(v) => write!(f, "{v}"),
            IntegerValue::Placeholder(v) => write!(f, "{v}"),
        }
    }
}
