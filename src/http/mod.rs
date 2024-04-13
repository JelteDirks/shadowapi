mod parsers;

use crate::http::parsers::parse_request_line;

pub use self::parsers::HttpError;
pub use self::parsers::HttpVersion;
pub use self::parsers::RequestLine;

use std::io::BufRead;

#[derive(Default)]
pub struct Unparsed;

pub struct HttpRequestBuilder {
    content: Vec<u8>,
}

pub struct HttpRequest {
}

impl HttpRequestBuilder {
    pub fn insert_bytes(&mut self, bytes: &[u8]) {
        self.content.extend_from_slice(bytes);
    }

    pub fn new() -> HttpRequestBuilder {
        HttpRequestBuilder {
            content: Default::default(),
        }
    }

    pub fn build(self) -> Result<HttpRequest, HttpError> {
        let lines: Vec<_> = self.content.as_slice()
            .lines()
            .collect();

        if lines[0].is_err() {
            return Err(HttpError::BadFormat);
        }

        let rl = lines[0].as_ref().unwrap();
        let rl = parse_request_line(rl.as_bytes())?;

        dbg!(rl);

        Err(HttpError::BadFormat)
    }
}
