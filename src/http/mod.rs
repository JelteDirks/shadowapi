use std::{collections::HashMap, fmt::Display};

/* Request Line grammar can be found here:
 * https://httpwg.org/specs/rfc9112.html#message.format
 */

#[derive(Debug)]
pub struct RawHttpRequest {
    bytes: Vec<u8>,
    size: usize,
}

impl Default for RawHttpRequest {
    fn default() -> Self {
        RawHttpRequest {
            bytes: Default::default(),
            size: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct DecodedHttpRequest {
    size: usize,
    method: HttpMethod,
    target: String,
    version: HttpVersion,
    headers: HashMap<String, String>,
}

impl RawHttpRequest {
    // Add bytes to the http request, return whether the http request is fully
    // parsed after this addition.
    pub fn add_bytes(&mut self, bytes: &[u8], n: usize) {
        self.bytes.extend_from_slice(&bytes);
        self.size = self.size + n;
    }

    pub fn decode(self) -> Result<DecodedHttpRequest, HttpError> {
        // TODO: parsing could be done more efficiently.
        // e.g.: iterateover the bytes and find the spaces, when spaces are
        // found, do something with the parts in between.

        // NOTE: many assumptions are made here which will probably not hold
        // in a valid environment. Should add proper error handling.

        let mut cursor: usize = 0;
        let method: HttpMethod;
        let target: String;
        let version: HttpVersion;

        if self.bytes.len() > 1 {
            method = match self.bytes[0] {
                0x43 => Ok(HttpMethod::Connect),
                0x44 => Ok(HttpMethod::Delete),
                0x47 => Ok(HttpMethod::Get),
                0x48 => Ok(HttpMethod::Head),
                0x4F => Ok(HttpMethod::Options),
                0x50 => match self.bytes[1] {
                    0x4F => Ok(HttpMethod::Post),
                    0x55 => Ok(HttpMethod::Put),
                    _ => Err(HttpError::BadFormat),
                },
                0x54 => Ok(HttpMethod::Trace),
                _ => Err(HttpError::BadFormat),
            }?;

            let sp = self.bytes.iter().position(|&byte| byte == 0x20);

            if sp.is_some() {
                cursor = sp.unwrap() + 1;
            }
        } else {
            return Err(HttpError::BadFormat);
        }

        let next_sp = self.bytes[cursor..].iter().position(|&byte| byte == 0x20);

        if let Some(sp) = next_sp {
            let range = cursor..cursor + sp;
            target = std::str::from_utf8(&self.bytes[range])
                .expect("expect the request line to be valid")
                .into();

            cursor = cursor + sp + 1;
        } else {
            return Err(HttpError::BadFormat);
        }

        let next_cr = self.bytes[cursor..]
            .iter()
            .position(|&byte| byte == 0x0A)
            .expect("request line should fit entirely when testing");

        let range = cursor..cursor + next_cr;
        version = self.bytes[range].try_into()?;

        Ok(DecodedHttpRequest {
            size: self.size,
            method,
            target,
            version,
            headers: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3, /* TODO: maybe not support this? */
}

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

impl From<HttpMethod> for &str {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
        }
    }
}

impl TryFrom<&[u8]> for HttpVersion {
    type Error = HttpError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            return Err(HttpError::BadFormat);
        }

        match value[0..=4] {
            [0x48, 0x54, 0x54, 0x50, 0x2F] => match value[5..=7] {
                [0x31, 0x2E, 0x30] => Ok(HttpVersion::Http10),
                [0x31, 0x2E, 0x31] => Ok(HttpVersion::Http11),
                [0x32, _] => Ok(HttpVersion::Http2),
                [0x33, _] => Ok(HttpVersion::Http3),
                _ => Err(HttpError::UnknownVersion),
            },
            _ => Err(HttpError::BadFormat),
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

impl From<HttpVersion> for &str {
    fn from(value: HttpVersion) -> Self {
        match value {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
            HttpVersion::Http3 => "HTTP/3",
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn small_request_method() {
        let mut rq: RawHttpRequest = RawHttpRequest::default();
        let payload =
            "GET /api HTTP/1.1\nHost: localhost:1234\nUser-Agent: curl/8.4.0\nAccept: */*\n\n";
        let _ = rq.add_bytes(payload.as_bytes(), payload.len());
        let rq = rq.decode().expect("should be decodable");
        let method: HttpMethod = rq.method;
        let asstr: &str = method.into();
        assert_eq!(asstr, "GET");
    }

    #[test]
    fn small_request_version() {
        let mut rq: RawHttpRequest = RawHttpRequest::default();
        let payload =
            "GET /api HTTP/1.1\nHost: localhost:1234\nUser-Agent: curl/8.4.0\nAccept: */*\n\n";
        let _ = rq.add_bytes(payload.as_bytes(), payload.len());
        let rq = rq.decode().expect("should be decodable");
        let version: HttpVersion = rq.version;
        let asstr: &str = version.into();
        assert_eq!(asstr, "HTTP/1.1");
    }

    #[test]
    fn small_request_target() {
        let mut rq: RawHttpRequest = RawHttpRequest::default();
        let payload =
            "GET /api HTTP/1.1\nHost: localhost:1234\nUser-Agent: curl/8.4.0\nAccept: */*\n\n";
        let _ = rq.add_bytes(payload.as_bytes(), payload.len());
        let rq = rq.decode().expect("should be decodable");
        assert_eq!(rq.target, "/api");
    }
}
