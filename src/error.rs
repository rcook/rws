use std::sync::Arc;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone)]
pub enum AppError {
    User(String),
    Internal(&'static str, String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::User(message) => write!(f, "User({})", message),
            AppError::Internal(facility, message) => {
                write!(f, "Internal.{}({})", facility, message)
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::convert::From<AppError> for rlua::Error {
    fn from(error: AppError) -> Self {
        rlua::Error::ExternalError(Arc::new(error))
    }
}

impl std::convert::From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        internal_error("IO", error.to_string())
    }
}

impl std::convert::From<std::str::Utf8Error> for AppError {
    fn from(error: std::str::Utf8Error) -> Self {
        internal_error("Utf8", error.to_string())
    }
}

impl std::convert::From<std::string::FromUtf8Error> for AppError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        internal_error("Utf8", error.to_string())
    }
}

impl std::convert::From<sxd_document::parser::Error> for AppError {
    fn from(error: sxd_document::parser::Error) -> Self {
        internal_error("Xml", error.to_string())
    }
}

impl std::convert::From<sxd_xpath::ExecutionError> for AppError {
    fn from(error: sxd_xpath::ExecutionError) -> Self {
        internal_error("Xml", error.to_string())
    }
}

impl std::convert::From<sxd_xpath::ParserError> for AppError {
    fn from(error: sxd_xpath::ParserError) -> Self {
        internal_error("Xml", error.to_string())
    }
}

impl std::convert::From<which::Error> for AppError {
    fn from(error: which::Error) -> Self {
        internal_error("Which", error.to_string())
    }
}

impl std::convert::From<yaml_rust::ScanError> for AppError {
    fn from(error: yaml_rust::ScanError) -> Self {
        internal_error("Yaml", error.to_string())
    }
}

pub fn user_error<S>(message: S) -> AppError
where
    S: Into<String>,
{
    AppError::User(message.into())
}

pub fn user_error_result<T, S>(message: S) -> Result<T>
where
    S: Into<String>,
{
    Err(AppError::User(message.into()))
}

pub fn internal_error<S>(facility: &'static str, message: S) -> AppError
where
    S: Into<String>,
{
    AppError::Internal(facility, message.into())
}
