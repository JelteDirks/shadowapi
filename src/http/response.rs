pub struct RawHttpResponse {
    pub bytes: Vec<u8>,
    pub size: usize,
}

impl From<Vec<u8>> for RawHttpResponse {
    fn from(value: Vec<u8>) -> Self {
        RawHttpResponse {
            size: value.len(),
            bytes: value,
        }
    }
}
