mod parsers;

use crate::http::parsers::parse_request_line;

pub use self::parsers::HttpParserError;
pub use self::parsers::HttpVersion;
pub use self::parsers::RequestLine;

#[derive(Default)]
pub struct Unparsed;

pub struct HttpRequestBuilder {
    content: Vec<u8>,
}

pub struct HttpRequest;

impl HttpRequestBuilder {
    pub fn insert_bytes(&mut self, bytes: &[u8]) {
        self.content.extend_from_slice(bytes);
    }

    pub fn new() -> HttpRequestBuilder {
        HttpRequestBuilder {
            content: Default::default(),
        }
    }

    pub fn build(self) -> Result<HttpRequest, HttpParserError> {
        parse_request_line(&self.content.as_slice());
        Ok(HttpRequest)
    }
}
