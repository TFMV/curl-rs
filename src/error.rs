use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    HttpParse(String),
    Connection(String),
    Timeout,
    InvalidUrl(String),
    InvalidResponse(String),
    Io(String),
    Json(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpParse(s) => write!(f, "HTTP parse error: {}", s),
            Self::Connection(s) => write!(f, "Connection error: {}", s),
            Self::Timeout => write!(f, "Request timeout"),
            Self::InvalidUrl(s) => write!(f, "Invalid URL: {}", s),
            Self::InvalidResponse(s) => write!(f, "Invalid response: {}", s),
            Self::Io(s) => write!(f, "IO error: {}", s),
            Self::Json(s) => write!(f, "JSON error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::TimedOut {
            Self::Timeout
        } else {
            Self::Io(e.to_string())
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "json")]
pub use serde_json::Error as JsonError;
