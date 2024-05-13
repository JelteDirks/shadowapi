use super::error::HttpError;

#[derive(Debug)]
pub struct RawHttpResponse {
    pub bytes: Vec<u8>,
    pub size: usize,
}

#[derive(Debug)]
pub struct DecodedHttpResponse {}

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
        Ok(DecodedHttpResponse {})
    }
}
