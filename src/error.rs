pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone)]
pub enum AppError {
    User(String),
    System(&'static str, String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::User(message) => write!(f, "User({})", message),
            AppError::System(kind, message) => write!(f, "System.{}({})", kind, message),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::convert::From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::System("IO", error.to_string())
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
