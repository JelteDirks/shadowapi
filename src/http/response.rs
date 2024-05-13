use super::{
    error::HttpError,
    partials::{HttpStatusCode, HttpVersion},
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

        Ok(DecodedHttpResponse { version, status })
    }
}
