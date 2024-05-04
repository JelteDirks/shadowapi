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
