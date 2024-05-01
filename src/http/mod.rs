use std::{fmt::Display, io::BufRead};

use tokio::io::AsyncBufReadExt;

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
    BadFormat,
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
    cursor: usize,
    method: Option<HttpMethod>,
    target: Option<String>,
    version: Option<HttpVersion>,
}

impl HttpRequest {
    // Add bytes to the http request, return whether the http request is fully
    // parsed after this addition.
    pub fn add_bytes(&mut self, bytes: &[u8]) -> Result<bool, HttpError> {
        self.bytes.extend_from_slice(&bytes);

        // TODO: parsing could be done more efficiently.
        // e.g.: iterateover the bytes and find the spaces, when spaces are
        // found, do something with the parts in between.

        if self.method.is_none() {
            if self.bytes.len() > 1 {
                let method = match self.bytes[0] {
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
                    self.cursor = sp.unwrap() + 1;
                }

                self.method = Some(method);
            }
        }

        if self.method.is_some() && self.target.is_none() {
            let next_sp = self.bytes[self.cursor..]
                .iter()
                .position(|&byte| byte == 0x20);

            if let Some(sp) = next_sp {
                let range = self.cursor..self.cursor + sp;
                let target = std::str::from_utf8(&self.bytes[range]);

                if let Ok(t) = target {
                    self.target = Some(t.to_owned());
                } else {
                    return Err(HttpError::BadFormat);
                }

                self.cursor = self.cursor + sp;
            }
        }

        if self.target.is_some() && self.version.is_none() {
            let lines = self
                .bytes
                .split(|&byte| byte == 0x0A)
                .take(1)
                .flat_map(|first_line| first_line.split(|&byte| byte == 0x20).skip(2).take(1))
                .map(|version| version)
                .collect::<Vec<_>>();

            if lines.len() == 0 {
                return Err(HttpError::BadFormat);
            }

            let version: HttpVersion = lines[0].try_into()?;

            self.version = Some(version);
        }

        dbg!(&self);

        Ok(false)
    }
}

impl Default for HttpRequest {
    fn default() -> Self {
        HttpRequest {
            bytes: Default::default(),
            cursor: Default::default(),
            method: Default::default(),
            target: Default::default(),
            version: Default::default(),
        }
    }
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
