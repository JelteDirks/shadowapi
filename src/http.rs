#[derive(Debug)]
pub struct RequestWrapper {
    pub request_line: Option<RequestLine>,
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: HttpMethod,
    pub version: HttpVersion,
    pub path: Vec<u8>,
}

impl RequestLine {
    pub fn from_string(line: &String) -> Result<Self, ParseError> {
        let mut method = HttpMethod::Unknown;
        let mut version = HttpVersion::Unknown;
        let mut path = vec![];

        let enum_lines = line.split(' ').enumerate();

        for (i, chunk) in enum_lines {
            if i == 0 {
                method = match chunk {
                    "OPTIONS" => HttpMethod::Options,
                    "HEAD" => HttpMethod::Head,
                    "POST" => HttpMethod::Post,
                    "PUT" => HttpMethod::Put,
                    "DELETE" => HttpMethod::Delete,
                    "TRACE" => HttpMethod::Trace,
                    "GET" => HttpMethod::Get,
                    "PATCH" => HttpMethod::Patch,
                    _ => HttpMethod::Unknown,
                }
            } else if i == 1 {
                path = chunk.into();
            } else if i == 2 {
                version = match chunk.trim() {
                    "HTTP/1.0" => HttpVersion::Http10,
                    "HTTP/1.1" => HttpVersion::Http11,
                    "HTTP/2" => HttpVersion::Http2,
                    _ => HttpVersion::Unknown,
                }
            }
        }

        if version == HttpVersion::Unknown {
            return Err(ParseError::NotHttp);
        }

        if method == HttpMethod::Unknown {
            return Err(ParseError::MalFormedHttp);
        }

        return Ok(RequestLine {
            method,
            path,
            version,
        });
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Unknown,
}

impl HttpVersion {
    pub fn to_string(&self) -> String {
        match self {
            HttpVersion::Http10 => String::from("1.0"),
            HttpVersion::Http11 => String::from("1.1"),
            HttpVersion::Http2 => String::from("2.0"),
            _ => String::from("unknown"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Patch,
    Options,
    Post,
    Get,
    Delete,
    Put,
    Head,
    Trace,
    Unknown,
}

impl HttpMethod {
    pub fn to_string(&self) -> String {
        match self {
            HttpMethod::Patch => String::from("PATCH"),
            HttpMethod::Options => String::from("OPTIONS"),
            HttpMethod::Post => String::from("POST"),
            HttpMethod::Get => String::from("GET"),
            HttpMethod::Delete => String::from("DELETE"),
            HttpMethod::Put => String::from("PUT"),
            HttpMethod::Head => String::from("HEAD"),
            HttpMethod::Trace => String::from("TRACE"),
            _ => String::from("UNKNOWN"),
        }
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    BadRequestLine,
}

#[derive(Debug)]
pub enum ParseError {
    NotHttp,
    MalFormedHttp,
}


