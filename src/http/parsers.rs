use std::io::BufRead;

#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3, /* TODO: maybe not support this? */
}

#[derive(Debug)]
pub enum HttpParserError {
    BadFormat,
    BadRequestLine,
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
    type Error = HttpParserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        todo!();
    }
}

#[derive(Debug)]
pub struct RequestLine {
    method: HttpMethod,
    path: String,
    version: HttpVersion,
}

impl TryFrom<String> for RequestLine {
    type Error = HttpParserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(' ').collect();

        if parts.len() != 3 {
            return Err(HttpParserError::BadRequestLine);
        }

        let method: HttpMethod = parts[0].to_string().try_into().unwrap();

        todo!();
    }
}

pub fn parse_request_line(bytes: &[u8]) -> Result<RequestLine, HttpParserError> {
    let lines: Vec<_> = bytes.lines().collect();
    let first = lines.first();

    if let None = first {
        return Err(HttpParserError::BadRequestLine);
    }

    let first = first.unwrap();

    return match first {
        Ok(requestline) => requestline.to_string().try_into(),
        Err(_) => Err(HttpParserError::BadRequestLine),
    };
}

// Request Line grammar can be found here:
// https://datatracker.ietf.org/doc/html/rfc2616#section-5
