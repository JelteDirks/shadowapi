use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3, /* TODO: maybe not support this? */
}

#[derive(Debug)]
pub enum HttpError {
    ParserError,
    BadFormat,
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



impl TryFrom<String> for HttpMethod {
    type Error = HttpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        todo!();
    }
}

#[derive(Debug)]
pub struct RequestLine {
    method: HttpMethod,
    uri: String,
    version: HttpVersion,
}

impl TryFrom<String> for RequestLine {
    type Error = HttpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(' ').collect();

        if parts.len() != 3 {
            return Err(HttpError::ParserError);
        }

        let method: HttpMethod = parts[0]
            .to_string()
            .try_into()
            .unwrap();

        todo!();
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::ParserError => write!(f, "Error while parsing the request"),
            HttpError::BadFormat => write!(f, "The request is badly formatted"),
        }
    }
}
impl Error for HttpError { }

// Request Line grammar can be found here:
pub fn parse_request_line(bytes: &[u8]) -> Result<RequestLine, HttpError> {

    let chunks: Vec<_> = bytes
        .split(|byte| *byte == b' ')
        .collect();

    if chunks.len() != 3 {
        return Err(HttpError::BadFormat);
    }

    let method = match chunks[0] {
        b"OPTIONS" => HttpMethod::Options,
        b"GET" => HttpMethod::Get,
        b"HEAD" => HttpMethod::Head,
        b"POST" => HttpMethod::Post,
        b"PUT" => HttpMethod::Put,
        b"DELETE" => HttpMethod::Delete,
        b"TRACE" => HttpMethod::Trace,
        b"CONNECT" => HttpMethod::Connect,
        _ => panic!("header unknown"),
    };

        let uri = String::from_utf8(chunks[1].to_vec())
            .expect("uri should be utf8");

    let version = match chunks[2] {
        b"HTTP/1.0" => HttpVersion::Http10,
        b"HTTP/1.1" => HttpVersion::Http11,
        b"HTTP/2" => HttpVersion::Http2,
        b"HTTP/3" => HttpVersion::Http3,
        _ => panic!("version unknown"),
    };

    Ok(RequestLine { method, uri, version })
}

// https://datatracker.ietf.org/doc/html/rfc2616#section-5
