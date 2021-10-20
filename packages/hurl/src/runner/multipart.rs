/*
 * hurl (https://hurl.dev)
 * Copyright (C) 2020 Orange
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
extern crate libxml;

use std::collections::HashMap;
use std::ffi::OsStr;
#[allow(unused)]
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;

use crate::http;
use crate::runner::DirectoryContext;
use hurl_core::ast::*;

use super::core::{Error, RunnerError};
use super::template::eval_template;
use super::value::Value;

pub fn eval_multipart_param<R: Read, D: DirectoryContext<R>>(
    multipart_param: MultipartParam,
    variables: &HashMap<String, Value>,
    context_dir: &D,
) -> Result<http::MultipartParam, Error> {
    match multipart_param {
        MultipartParam::Param(KeyValue { key, value, .. }) => {
            let name = key.value;
            let value = eval_template(value, variables)?;
            Ok(http::MultipartParam::Param(http::Param { name, value }))
        }
        MultipartParam::FileParam(param) => {
            let file_param = eval_file_param(param, context_dir)?;
            Ok(http::MultipartParam::FileParam(file_param))
        }
    }
}

pub fn eval_file_param<R: Read, D: DirectoryContext<R>>(
    file_param: FileParam,
    context_dir: &D,
) -> Result<http::FileParam, Error> {
    let name = file_param.key.value;

    let filename = file_param.value.filename.clone();

    let data = match context_dir.open(&filename) {
        Ok(mut f) => {
            let mut bytes = Vec::new();
            match f.read_to_end(&mut bytes) {
                Ok(_) => bytes,
                Err(_) => {
                    return Err(Error {
                        source_info: filename.source_info.clone(),
                        inner: RunnerError::FileReadAccess {
                            value: context_dir.get_absolute_filename(&filename),
                        },
                        assert: false,
                    });
                }
            }
        }
        Err(_) => {
            return Err(Error {
                source_info: filename.source_info.clone(),
                inner: RunnerError::FileReadAccess {
                    value: context_dir.get_absolute_filename(&filename),
                },
                assert: false,
            });
        }
    };

    if !context_dir.exists(&filename) {
        return Err(Error {
            source_info: filename.source_info,
            inner: RunnerError::FileReadAccess {
                value: filename.value.clone(),
            },
            assert: false,
        });
    }

    let content_type = file_value_content_type(file_param.value);
    Ok(http::FileParam {
        name,
        filename: filename.value,
        data,
        content_type,
    })
}

pub fn file_value_content_type(file_value: FileValue) -> String {
    match file_value.content_type.clone() {
        None => match Path::new(file_value.filename.value.as_str())
            .extension()
            .and_then(OsStr::to_str)
        {
            Some("gif") => "image/gif".to_string(),
            Some("jpg") => "image/jpeg".to_string(),
            Some("jpeg") => "image/jpeg".to_string(),
            Some("png") => "image/png".to_string(),
            Some("svg") => "image/svg+xml".to_string(),
            Some("txt") => "text/plain".to_string(),
            Some("htm") => "text/html".to_string(),
            Some("html") => "text/html".to_string(),
            Some("pdf") => "application/pdf".to_string(),
            Some("xml") => "application/xml".to_string(),
            _ => "application/octet-stream".to_string(),
        },
        Some(content_type) => content_type,
    }
}

#[cfg(test)]
mod tests {
    use hurl_core::ast::SourceInfo;

    use super::super::FsDirectoryContext;
    use super::*;

    pub fn whitespace() -> Whitespace {
        Whitespace {
            value: String::from(" "),
            source_info: SourceInfo::init(0, 0, 0, 0),
        }
    }

    #[test]
    pub fn test_eval_file_param() {
        let line_terminator = LineTerminator {
            space0: whitespace(),
            comment: None,
            newline: whitespace(),
        };
        assert_eq!(
            eval_file_param(
                FileParam {
                    line_terminators: vec![],
                    space0: whitespace(),
                    key: EncodedString {
                        value: "upload1".to_string(),
                        encoded: "upload1".to_string(),
                        quotes: false,
                        source_info: SourceInfo::init(0, 0, 0, 0),
                    },
                    space1: whitespace(),
                    space2: whitespace(),
                    value: FileValue {
                        space0: whitespace(),
                        filename: Filename {
                            value: "hello.txt".to_string(),
                            source_info: SourceInfo::init(0, 0, 0, 0),
                        },
                        space1: whitespace(),
                        space2: whitespace(),
                        content_type: None,
                    },
                    line_terminator0: line_terminator,
                },
                &FsDirectoryContext::new("tests".to_string()),
            )
            .unwrap(),
            http::FileParam {
                name: "upload1".to_string(),
                filename: "hello.txt".to_string(),
                data: b"Hello World!".to_vec(),
                content_type: "text/plain".to_string(),
            }
        );
    }

    #[test]
    pub fn test_file_value_content_type() {
        assert_eq!(
            file_value_content_type(FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello.txt".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: None,
            }),
            "text/plain".to_string()
        );

        assert_eq!(
            file_value_content_type(FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello.html".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: None,
            }),
            "text/html".to_string()
        );

        assert_eq!(
            file_value_content_type(FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello.txt".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: Some("text/html".to_string()),
            }),
            "text/html".to_string()
        );

        assert_eq!(
            file_value_content_type(FileValue {
                space0: whitespace(),
                filename: Filename {
                    value: "hello".to_string(),
                    source_info: SourceInfo::init(0, 0, 0, 0),
                },
                space1: whitespace(),
                space2: whitespace(),
                content_type: None,
            }),
            "application/octet-stream".to_string()
        );
    }
}
