use super::{
    error::HttpError,
    partials::{HttpHeader, HttpHeaderPair, HttpStatusCode, HttpVersion},
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


pub fn build_header_pair(buf: &[u8], l: usize, r: usize, h: HttpHeader) -> Option<HttpHeaderPair> {
    let mut start: usize = l;
    let mut end: usize = r;

    for p in l..=r {
        if (buf[p] as char).is_whitespace() {
            start += 1;
        } else { break; }
    }

    for p in r..=l {
        if (buf[p] as char).is_whitespace() {
            end -= 1;
        } else { break; }
    }

    if start >= end {
        return None;
    }

    return Some((h, String::from_utf8(buf[start..end].to_vec()).unwrap()));
}

pub fn len_match(buf: &[u8], s: usize, e: usize, offset: usize) -> bool {
    return buf[s+offset] == b':' && s+offset < e;
}

// buf: ref to the raw bytes
// s: start of the current header line
// e: end of the current header line (points to the \n)
pub fn decode_header(buf: &[u8], s: usize, e: usize) -> Option<HttpHeaderPair> {
    // NOTE: end is including the position of \n, in http there are CRLF line
    // endings which mean there is a \r before \n at position [end - 1].

    match buf[s] {
        b'A' => { /* A */
            match buf[s + 1] {
                b'c' => { /* Ac */
                    match buf[s + 4] {
                        b'p' => { /* Ac[ce]p */
                            if len_match(buf,s,e,6) {
                                return build_header_pair(buf, s+7, e-1, HttpHeader::Accept);
                            }
                            match buf[s + 7] {
                                b'P' => { /* Ac[ce]p[t-]P */
                                    if len_match(buf,s,e,12) {
                                        return build_header_pair(buf, s+8, e-1, HttpHeader::AcceptPatch);
                                    }
                                }
                                b'R' => { /* Ac[ce]p[t-]R */
                                    if len_match(buf,s,e,12) {
                                        return build_header_pair(buf, s+8, e-1, HttpHeader::AcceptRanges);
                                    }
                                }
                                _ => {}
                            }
                        }
                        b's' => { /* Ac[ce]s */
                            match buf[s+15] {
                                b'A' => { /* Access-Control-A */
                                    match buf[s+22] {
                                        b'O' => { /* Access-Control-Allow-O */
                                            if len_match(buf,s,e,27) {
                                                return build_header_pair(buf, s+28, e-1, HttpHeader::AccessControlAllowOrigin);
                                            }
                                        }
                                        b'C' => {
                                            if len_match(buf,s,e,32) {
                                                return build_header_pair(buf, s+33, e-1, HttpHeader::AccessControlAllowCredentials);
                                            }
                                        }
                                        b'M' => {
                                            if len_match(buf,s,e,28) {
                                                return build_header_pair(buf, s+29, e-1, HttpHeader::AccessControlAllowMethods);
                                            }
                                        }
                                        b'H' => {
                                            if len_match(buf,s,e,28) {
                                                return build_header_pair(buf, s+29, e-1, HttpHeader::AccessControlAllowHeaders);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                b'E' => {
                                    if len_match(buf,s,e,29) {
                                        return build_header_pair(buf, s+30, e-1, HttpHeader::AccessControlExposeHeaders);
                                    }
                                }
                                b'M' => {
                                    if len_match(buf,s,e,22) {
                                        return build_header_pair(buf, s+23, e-1, HttpHeader::AccessControlMaxAge);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                b'g' => { /* Ag */
                    if len_match(buf, s, e, 3) {
                        return build_header_pair(buf, s + 4, e - 1, HttpHeader::Age);
                    }
                }
                b'l' => { /* Al */
                    match buf[s+2] {
                        b'l' => { /* All */
                            if len_match(buf,s,e,5) {
                                return build_header_pair(buf, s+6, e-1, HttpHeader::Allow);
                            }
                        }
                        b't' => { /* Alt */
                            if len_match(buf,s,e,7) {
                                return build_header_pair(buf, s+8, e-1, HttpHeader::AltSvc);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        b'C' => { /* C */
            match buf[s + 1] {
                b'a' => { /* Ca */
                    if len_match(buf,s,e,13) {
                        return build_header_pair(buf, s+14, e-1, HttpHeader::CacheControl);
                    }
                }
                b'o' => { /* Co */
                    match buf[s + 3] {
                        b'n' => { /* Con */
                            if len_match(buf,s,e,10) { // TODO: deal with open conns
                                return build_header_pair(buf, s+11, e-1, HttpHeader::Connection);
                            }
                        }
                        b't' => { /* Cont */
                            match buf[s + 8] {
                                b'D' => { /* Content-D */
                                    if len_match(buf,s,e,19) {
                                        return build_header_pair(buf, s+20, e-1, HttpHeader::ContentDisposition);
                                    }
                                }
                                b'E' => { /* Content-E */
                                    if len_match(buf,s,e,16) {
                                        return build_header_pair(buf, s+17, e-1, HttpHeader::ContentEncoding);
                                    }
                                }
                                b'L' => { /* Content-L */
                                    match buf[s+ 9] {
                                        b'a' => { /* Content-La */
                                            if len_match(buf,s,e,16) {
                                                return build_header_pair(buf, s+17, e-1, HttpHeader::ContentLanguage);
                                            }
                                        }
                                        b'e' => { /* Content-Le */
                                            if len_match(buf,s,e,14) {
                                                return build_header_pair(buf, s + 15, e - 1, HttpHeader::ContentLength);
                                            }
                                        }
                                        b'o' => { /* Content-Lo */
                                            if len_match(buf,s,e,16) {
                                                return build_header_pair(buf, s+17, e-1, HttpHeader::ContentLocation);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                b'R' => { /* Content-R */
                                    if len_match(buf,s,e,13) {
                                        return build_header_pair(buf, s+14, e-1, HttpHeader::ContentRange);
                                    }
                                }
                                b'T' => { /* Content-T */
                                    if len_match(buf,s,e,12) {
                                        return build_header_pair(buf, s+13, e-1, HttpHeader::ContentType);
                                    }
                                }
                                b'S' => { /* Content-S */
                                    if len_match(buf,s,e,23) {
                                        return build_header_pair(buf, s+24, e-1, HttpHeader::ContentSecurityPolicy);
                                    }
                                }
                                _ => {
                                }
                            }
                        }
                        _ => {
                        }
                    }
                }
                _ => {
                }
            }
        }
        b'D' => { /* D */
            match buf[s+1] {
                b'a' => { /* Da */
                    if len_match(buf,s,e,4) {
                        return build_header_pair(buf, s+5, e-1, HttpHeader::Date);
                    }
                }
                b'e' => { /* De */
                    if len_match(buf,s,e,10) {
                        return build_header_pair(buf, s+11, e-1, HttpHeader::DeltaBase);
                    }
                }
                _ => {}
            }
        }
        b'E' => { /* E */
            match buf[s+1] {
                b'T' => { /* ET */
                    if len_match(buf,s,e,4) {
                        return build_header_pair(buf, s+5, e-1, HttpHeader::ETag);
                    }
                }
                b'x' => { /* Ex */
                    if len_match(buf,s,e,7) {
                        return build_header_pair(buf, s+8, e-1, HttpHeader::Expires);
                    }
                }
                _ => {}
            }
        }
        b'I' => { /* I */
            if len_match(buf,s,e,2) {
                return build_header_pair(buf, s+3, e-1, HttpHeader::IM);
            }
        }
        b'L' => { /* L */
            match buf[s+1] {
                b'a' => { /* La */
                    if len_match(buf,s,e,13) {
                        return build_header_pair(buf, s+14, e-1, HttpHeader::LastModified);
                    }
                }
                b'i' => { /* Li */
                    if len_match(buf,s,e,4) {
                        return build_header_pair(buf, s+5, e-1, HttpHeader::Link);
                    }
                }
                b'o' => {
                    if len_match(buf,s,e,8) {
                        return build_header_pair(buf, s+9, e-1, HttpHeader::Location);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    return None;
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
            Some(n) => n,
            None => {
                return Ok(DecodedHttpResponse { version, status });
            }
        };

        // TODO: decode headers
        let mut cursor = next_lf;
        loop {
            let next_lf = self.bytes[cursor..].iter().position(|&byte| byte == 0x0A);

            if next_lf.is_none() {
                break;
            }

            let line_length = next_lf.unwrap();

            let header_pair = decode_header(&self.bytes, cursor, cursor + line_length);
            if header_pair.is_some() {
                println!("{:?}", header_pair);
            }

            cursor = cursor + line_length + 1;
        }

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
