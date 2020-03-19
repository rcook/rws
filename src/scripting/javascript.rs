use crate::error::{internal_error, AppError, Result};

use super::variables::Variables;

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
    _variables: &Variables,
) -> Result<T> {
    unimplemented!("JavaScript not implemented yet!")
}
