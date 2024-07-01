use super::{
    error::HttpError,
    partials::{HttpHeaderPair, HttpStatusCode, HttpVersion},
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

pub fn decode_header() -> HttpHeaderPair {

    todo!();
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

        let cursor = next_sp + 4;

        // TODO: decode headers
        let next_lf = self.bytes[cursor..].iter().position(|&byte| byte == 0x0A);
        let cursor = if let Some(n) = next_lf {
            cursor + n + 1
        } else {
            return Ok(DecodedHttpResponse { version, status });
        };

        dbg!(String::from_utf8(self.bytes[cursor..].to_vec()).unwrap());

        Ok(DecodedHttpResponse { version, status })
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
