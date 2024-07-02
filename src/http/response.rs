use super::{
    error::HttpError,
    partials::{HttpHeader, HttpHeaderPair, HttpStatusCode, HttpVersion},
    decoders::*
};

#[derive(Debug)]
pub struct RawHttpResponse {
    pub bytes: Vec<u8>,
    pub size: usize,
}

#[derive(Debug)]
pub struct DecodedHttpResponse {
    pub version: HttpVersion,
    pub status: HttpStatusCode,
    pub headers: Vec<HttpHeaderPair>,
    pub content_length: Option<usize>,
}

impl From<Vec<u8>> for RawHttpResponse {
    fn from(value: Vec<u8>) -> Self {
        RawHttpResponse {
            size: value.len(),
            bytes: value,
        }
    }
}

impl Default for RawHttpResponse {
    fn default() -> Self {
        RawHttpResponse {
            size: Default::default(),
            bytes: Default::default(),
        }
    }
}

impl Default for DecodedHttpResponse {
    fn default() -> Self {
        todo!();
    }
}


impl RawHttpResponse {
    pub fn decode(self) -> Result<DecodedHttpResponse, HttpError> {
        let next_sp = self.bytes.iter().position(|&byte| byte == 0x20);
        if next_sp.is_none() {
            return Err(HttpError::BadFormat);
        }
        let next_sp = next_sp.unwrap();

        let version: Result<HttpVersion, _> = self.bytes[0..next_sp].try_into();
        if version.is_err() {
            return Err(HttpError::BadFormat);
        }
        let version = version.unwrap();
        let range = next_sp + 1..next_sp + 4;
        let status: HttpStatusCode = self.bytes[range].into();

        let next_lf = match self.bytes[next_sp + 4..].iter().position(|&byte| byte == 0x0A) {
            Some(n) => next_sp + 4 + n,
            None => {
                return Ok(DecodedHttpResponse { version, status, headers: Vec::default(), content_length: Some(0)});
            }
        };

        let mut headers: Vec<HttpHeaderPair> = Vec::with_capacity(10);

        // TODO: decode headers
        let mut cursor = next_lf;
        loop {
            let next_lf = self.bytes[cursor..].iter().position(|&byte| byte == 0x0A);

            if next_lf.is_none() {
                break;
            }

            let line_length = next_lf.unwrap();

            if let Some(header_pair) = decode_header(&self.bytes, cursor, cursor + line_length) {
                headers.push(header_pair);
            }

            cursor = cursor + line_length + 1;
        }

        let mut content_length: Option<usize> = None;

        headers.iter().for_each(|header_pair| {
            if header_pair.0 == HttpHeader::ContentLength {
                if let Ok(n) = header_pair.1.parse::<usize>() {
                    content_length = Some(n);
                }
            }
        });

        Ok(DecodedHttpResponse { version, status, headers, content_length })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with_msg() {
        let payload = "HTTP/1.1 200 OK";
        let raw: RawHttpResponse = RawHttpResponse::from(Vec::from(payload));
        let actual = raw.decode().expect("decoding should work");

        assert_eq!(actual.version, HttpVersion::Http11);
        assert_eq!(actual.status, HttpStatusCode::Ok200);
    }

    #[test]
    fn no_msg_response() {
        let payload = "HTTP/1.1 200";
        let raw: RawHttpResponse = RawHttpResponse::from(Vec::from(payload));
        let actual = raw.decode().expect("decoding should work");

        assert_eq!(actual.version, HttpVersion::Http11);
        assert_eq!(actual.status, HttpStatusCode::Ok200);
    }
}
