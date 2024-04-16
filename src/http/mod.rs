use std::fmt::Display;

/* Request Line grammar can be found here:
 * https://httpwg.org/specs/rfc9112.html#message.format
 */

#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3, /* TODO: maybe not support this? */
}

#[derive(Debug)]
pub enum HttpError {
    UnknownVersion,
}

#[derive(Debug)]
pub enum HttpMethod {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
}

#[derive(Debug)]
pub struct HttpRequest {
    bytes: Vec<u8>,
}

impl HttpRequest {
    // Add bytes to the http request, return whether the http request is fully
    // parsed after this addition.
    pub fn add_bytes(&mut self, bytes: &[u8]) -> Result<bool, HttpError> {
        self.bytes.extend_from_slice(&bytes);
        Ok(false)
    }
}

impl Default for HttpRequest {
    fn default() -> Self {
        HttpRequest {
            bytes: Default::default(),
        }
    }
}

impl std::error::Error for HttpError {}
impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::UnknownVersion => write!(f, "Http version seems to be unknown"),
        }
    }
}

impl TryFrom<&str> for HttpVersion {
    type Error = HttpError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "HTTP/1.0" => Ok(HttpVersion::Http10),
            "HTTP/1.1" => Ok(HttpVersion::Http11),
            "HTTP/2" => Ok(HttpVersion::Http2),
            "HTTP/3" => Ok(HttpVersion::Http3),
            _ => Err(HttpError::UnknownVersion),
        }
    }
}

impl TryFrom<&str> for HttpMethod {
    type Error = HttpError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "OPTIONS" => Ok(HttpMethod::Options),
            "GET" => Ok(HttpMethod::Get),
            "HEAD" => Ok(HttpMethod::Head),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "TRACE" => Ok(HttpMethod::Trace),
            "CONNECT" => Ok(HttpMethod::Connect),
            _ => Err(HttpError::UnknownVersion),
        }
    }
}
