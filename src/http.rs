use std::string::FromUtf8Error;

#[derive(Default)]
pub struct Unparsed;

#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3, /* TODO: maybe not support this? */
}

pub enum HttpError {
    BadFormat,
}

#[derive(Debug)]
pub struct HttpRequest {
    method: String,
    path: String,
    version: HttpVersion,
}

pub struct HttpRequestBuilder {
    content: Vec<u8>,
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
        // Parse here
        Ok(HttpRequest {
            method: Default::default(),
            path: Default::default(),
            version: HttpVersion::Http11,
        })
    }
}

impl TryFrom<HttpRequestBuilder> for String {
    type Error = FromUtf8Error;
    fn try_from(value: HttpRequestBuilder) -> Result<Self, Self::Error> {
        String::from_utf8(value.content)
    }
}
