use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum HttpError {
    BadFormat,
    UnknownVersion,
}

impl std::error::Error for HttpError {}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::UnknownVersion => write!(f, "Http version seems to be unknown"),
            HttpError::BadFormat => write!(f, "Http seems to be wrongly formatted"),
        }
    }
}

#[derive(Debug)]
pub enum ServerError {
    Unresponsive(String, Box<dyn Error + Send + Sync>),
    ServerWriteError(String, Box<dyn Error + Send + Sync>),
    ServerReadError(String, Box<dyn Error + Send + Sync>),
}
impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerReadError(target, err) => {
                write!(f, "server [{target}] error while reading, reason: {err}")
            }
            Self::ServerWriteError(target, err) => {
                write!(f, "server [{target}] error while writing, reason: {err}")
            }
            Self::Unresponsive(target, err) => {
                write!(f, "server [{target}] is unresponsive, reason: {err}")
            }
        }
    }
}
impl std::error::Error for ServerError {}
