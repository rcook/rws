use crate::config::ConfigObject;
use crate::error::{internal_error, AppError, Result};

pub trait Evaluatable: Default {}
impl<T: Default> Evaluatable for T {}

enum Error {}

impl std::convert::From<Error> for AppError {
    fn from(_error: Error) -> Self {
        internal_error("JavaScript", "(no message)")
    }
}

pub fn eval<T: Evaluatable>(
    _script: &str,
    _use_prelude: bool,
    _variables: &Vec<(String, ConfigObject)>,
) -> Result<T> {
    unimplemented!("JavaScript not implemented yet!")
}
