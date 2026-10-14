use serde::Serialize;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub code: &'static str,
    pub message: Box<str>,
    pub field: Option<Box<str>>,
    pub retryable: bool,
    pub corrective_argv: Vec<String>,
    pub request_id: Option<String>,
    pub retry_after_seconds: Option<u64>,
    #[serde(skip)]
    pub exit: u8,
}

impl Error {
    pub fn new(code: &'static str, message: impl Into<String>, exit: u8) -> Self {
        Self {
            code,
            message: message.into().into_boxed_str(),
            field: None,
            retryable: false,
            corrective_argv: vec![],
            request_id: None,
            retry_after_seconds: None,
            exit,
        }
    }
    pub fn input(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message, 2)
    }
    pub fn io(error: std::io::Error) -> Self {
        Self::new("local_io", error.to_string(), 3)
    }
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new("outcome_unknown", message, 7)
    }
    pub fn limit(message: impl Into<String>) -> Self {
        Self::new("limit_exceeded", message, 6)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::input("Invalid JSON document")
    }
}
