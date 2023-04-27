// The MIT License (MIT)
//
// Copyright (c) 2020-3 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    User(String),
    Internal(&'static str, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::User(message) => write!(f, "User({})", message),
            Error::Internal(facility, message) => write!(f, "Internal.{}({})", facility, message),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::convert::From<Error> for rlua::Error {
    fn from(error: Error) -> Self {
        rlua::Error::ExternalError(Arc::new(error))
    }
}

impl std::convert::From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        internal_error("Git", error.to_string())
    }
}

impl std::convert::From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        internal_error("Regex", error.to_string())
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        internal_error("IO", error.to_string())
    }
}

impl std::convert::From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        internal_error("Utf8", error.to_string())
    }
}

impl std::convert::From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        internal_error("Utf8", error.to_string())
    }
}

impl std::convert::From<sxd_document::parser::Error> for Error {
    fn from(error: sxd_document::parser::Error) -> Self {
        internal_error("Xml", error.to_string())
    }
}

impl std::convert::From<sxd_xpath::ExecutionError> for Error {
    fn from(error: sxd_xpath::ExecutionError) -> Self {
        internal_error("Xml", error.to_string())
    }
}

impl std::convert::From<sxd_xpath::ParserError> for Error {
    fn from(error: sxd_xpath::ParserError) -> Self {
        internal_error("Xml", error.to_string())
    }
}

impl std::convert::From<which::Error> for Error {
    fn from(error: which::Error) -> Self {
        internal_error("Which", error.to_string())
    }
}

impl std::convert::From<yaml_rust::ScanError> for Error {
    fn from(error: yaml_rust::ScanError) -> Self {
        internal_error("Yaml", error.to_string())
    }
}

pub fn user_error<S>(message: S) -> Error
where
    S: Into<String>,
{
    Error::User(message.into())
}

pub fn user_error_result<T, S>(message: S) -> Result<T>
where
    S: Into<String>,
{
    Err(Error::User(message.into()))
}

pub fn internal_error<S>(facility: &'static str, message: S) -> Error
where
    S: Into<String>,
{
    Error::Internal(facility, message.into())
}
