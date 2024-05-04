use std::collections::HashMap;

use crate::http::error::HttpError;
use crate::http::partials::{HttpMethod, HttpVersion};

/* Request Line grammar can be found here:
 * https://httpwg.org/specs/rfc9112.html#message.format
 */

#[derive(Debug)]
pub struct DecodedHttpRequest {
    size: usize,
    method: HttpMethod,
    target: String,
    version: HttpVersion,
    headers: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct RawHttpRequest {
    bytes: Vec<u8>,
    size: usize,
}

impl RawHttpRequest {
    // Add bytes to the http request, return whether the http request is fully
    // parsed after this addition.
    pub fn add_bytes(&mut self, bytes: &[u8], n: usize) {
        self.bytes.extend_from_slice(bytes);
        self.size += n;
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

            if let Some(sp) = self.bytes.iter().position(|&byte| byte == 0x20) {
                cursor = sp + 1;
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
        let version: HttpVersion = self.bytes[range].try_into()?;

        Ok(DecodedHttpRequest {
            size: self.size,
            method,
            target,
            version,
            headers: HashMap::new(),
        })
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
