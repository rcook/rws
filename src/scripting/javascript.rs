use crate::error::{internal_error, Error, Result};

use super::variables::Variables;

pub trait Evaluatable: Default {}
impl<T: Default> Evaluatable for T {}

enum JavaScriptError {}

impl std::convert::From<JavaScriptError> for Error {
    fn from(_error: JavaScriptError) -> Self {
        internal_error("JavaScript", "(no message)")
    }
}

pub fn eval<T: Evaluatable>(
    _preamble: &str,
    _script: &str,
    _use_prelude: bool,
    _variables: &Variables,
) -> Result<T> {
    unimplemented!("JavaScript not implemented yet!")
}
