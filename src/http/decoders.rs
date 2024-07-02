
// Builds a header pair for the given inputs. Whitespace is trimmed on both ends of the buffer.
// Note that there are no checks whether these values actually make sense, this should be done
// before calling this function.
//
// buf: a reference to a byte slice
// l: the left side of the value to be parsed
// r: the right side of the value to be parsed
// h: the name of the header
use crate::http::partials::HttpHeader;
use crate::http::partials::HttpHeaderPair;

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

// Checks if the length of a header is correctly set based on the expected offset and the position
// of the ':' character. If it is lower than the end of the header, it means that the ':' character
// was not at the expected offset as well. This check is needed since the buffer can exceed the
// length of one header as it is a byte slice ref.
//
// buf: reference to the byte slice
// s: the start position of the current header
// e: the end position of the current header (the next \n char)
// offset: the offset that the ':' character should be expected (and checked)
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
        b'P' => { /* P */
            match (buf[s+1], buf[s+2]) {
                (b'r', b'a') => { /* Pra */
                    if len_match(buf,s,e,6) {
                        return build_header_pair(buf, s+7, e-1, HttpHeader::Pragma);
                    }
                }
                (b'r', b'o') => { /* Pro */
                    if len_match(buf,s,e,18) {
                        return build_header_pair(buf, s+19, e-1, HttpHeader::ProxyAuthenticate);
                    }
                }
                _ => {}
            }
            match buf[s+1] {
                b'u' => {
                    if len_match(buf,s,e,15) {
                        return build_header_pair(buf, s+16, e-1, HttpHeader::PublicKeyPins);
                    }
                }
                _ => {}
            }
        }
        b'R' => { /* R */
            match buf[s+2] {
                b't' => { /* Ret */
                    if len_match(buf,s,e,11) {
                        return build_header_pair(buf, s+12, e-1, HttpHeader::RetryAfter);
                    }
                }
                b'f' => { /* Ref */
                    if len_match(buf,s,e,7) {
                        return build_header_pair(buf, s+8, e-1, HttpHeader::Refresh);
                    }
                }
                _ => {}
            }
        }
        b'S' => { /* S */
            match buf[s+1] {
                b'e' => { /* Se */
                    match buf[s+2] {
                        b'r' => { /* Ser */
                            if len_match(buf,s,e,6) {
                                return build_header_pair(buf, s+7, e-1, HttpHeader::Server);
                            }
                        }
                        b't' => { /* Set */
                            if len_match(buf,s,e,10) {
                                return build_header_pair(buf, s+11, e-1, HttpHeader::SetCookie);
                            }
                        }
                        _ => {}
                    }
                }
                b't' => { /* St */
                    if len_match(buf,s,e,25) {
                        return build_header_pair(buf, s+26, e-1, HttpHeader::StrictTransportSecurity);
                    }
                }
                _ => {}
            }
        }
        b'T' => { /* T */
            match buf[s+1] {
                b'r' => { /* Tr */
                    match buf[s+3] {
                        b'i' => { /* Trai */
                            if len_match(buf,s,e,7) {
                                return build_header_pair(buf, s+8, e-1, HttpHeader::Trailer);
                            }
                        }
                        b'n' => { /* Tran */
                            if len_match(buf,s,e,17) {
                                return build_header_pair(buf, s+18, e-1, HttpHeader::TransferEncoding);
                            }
                        }
                        _ => {}
                    }
                }
                b'k' => { /* Tk */
                    if len_match(buf,s,e,2) {
                        return build_header_pair(buf, s+3, e-1, HttpHeader::Tk);
                    }
                }
                _ => {}
            }
        }
        b'U' => { /* U */
            if len_match(buf,s,e,7) {
                return build_header_pair(buf, s+8, e-1, HttpHeader::Upgrade);
            }
        }
        b'V' => { /* V */
            match buf[s+1] {
                b'a' => { /* Va */
                    if len_match(buf,s,e,4) {
                        return build_header_pair(buf, s+5, e-1, HttpHeader::Vary);
                    }
                }
                b'i' => { /* Vi */
                    if len_match(buf,s,e,3) {
                        return build_header_pair(buf, s+4, e-1, HttpHeader::Via);
                    }
                }
                _ => {}
            }
        }
        b'W' => { /* W */
            match buf[s+1] {
                b'a' => { /* Wa */
                    if len_match(buf,s,e,7) {
                        return build_header_pair(buf, s+8, e-1, HttpHeader::Warning);
                    }
                }
                b'W' => { /* WW */
                    if len_match(buf,s,e,16) {
                        return build_header_pair(buf, s+17, e-1, HttpHeader::WWWAuthenticate);
                    }
                }
                _ => {}
            }
        }
        b'X' => { /* X */
            match buf[s+2] {
                b'P' => { /* X-P */
                    if len_match(buf,s,e,12) {
                        return build_header_pair(buf, s+13, e-1, HttpHeader::XPoweredBy);
                    }
                }
                b'R' => { /* X-R */
                    if len_match(buf,s,e,12) {
                        return build_header_pair(buf, s+13, e-1, HttpHeader::XRequestID);
                    }
                }
                b'U' => { /* X-U */
                    if len_match(buf,s,e,15) {
                        return build_header_pair(buf, s+16, e-1, HttpHeader::XUACompatible);
                    }
                }
                b'X' => { /* X-X */
                    if len_match(buf,s,e,16) {
                        return build_header_pair(buf, s+17, e-1, HttpHeader::XXSSProtection);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    return None;
}
