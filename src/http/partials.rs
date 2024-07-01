use crate::http::error::*;

#[derive(Debug, PartialEq)]
pub enum HttpHeader {
    AcceptPatch,
    AcceptRanges,
    AccessControlAllowOrigin,
    AccessControlAllowCredentials,
    AccessControlAllowMethods,
    AccessControlAllowHeaders,
    AccessControlExposeHeaders,
    AccessControlMaxAge,
    Age,
    Allow,
    AltSvc,
    CacheControl,
    Connection,
    ContentDisposition,
    ContentEncoding,
    ContentLanguage,
    ContentLength,
    ContentLocation,
    ContentRange,
    ContentType,
    ContentSecurityPolicy,
    Date,
    DeltaBase,
    ETag,
    Expires,
    IM,
    LastModified,
    Link,
    Location,
    Pragma,
    ProxyAuthenticate,
    PublicKeyPins,
    RetryAfter,
    Refresh,
    Server,
    SetCookie,
    StrictTransportSecurity,
    Trailer,
    TransferEncoding,
    Tk,
    Upgrade,
    Vary,
    Via,
    Warning,
    WWWAuthenticate,
    XPoweredBy,
    XRequestID,
    XUACompatible,
    XXSSProtection,
}

pub type HttpHeaderPair = (HttpHeader,String);

#[derive(Debug, PartialEq)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3, /* TODO: maybe not support this? */
}

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
}

#[derive(Debug, PartialEq)]
pub enum HttpStatusCode {
    Ok200,
    Unknown,
}

impl From<&[u8]> for HttpStatusCode {
    fn from(value: &[u8]) -> Self {
        match value {
            [0x32, 0x30, 0x30] => HttpStatusCode::Ok200,
            _ => Self::Unknown,
        }
    }
}

impl From<HttpMethod> for &str {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Get => "GET",
            HttpMethod::Head => "HEAD",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
        }
    }
}

impl TryFrom<&[u8]> for HttpVersion {
    type Error = HttpError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            return Err(HttpError::BadFormat);
        }

        match value[0..=4] {
            [0x48, 0x54, 0x54, 0x50, 0x2F] => match value[5..=7] {
                [0x31, 0x2E, 0x30] => Ok(HttpVersion::Http10),
                [0x31, 0x2E, 0x31] => Ok(HttpVersion::Http11),
                [0x32, _] => Ok(HttpVersion::Http2),
                [0x33, _] => Ok(HttpVersion::Http3),
                _ => Err(HttpError::UnknownVersion),
            },
            _ => Err(HttpError::BadFormat),
        }
    }
}

impl TryFrom<&str> for HttpVersion {
    type Error = HttpError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "HTTP/1.0" => Ok(HttpVersion::Http10),
            "HTTP/1.1" => Ok(HttpVersion::Http11),
            "HTTP/2" => Ok(HttpVersion::Http2),
            "HTTP/3" => Ok(HttpVersion::Http3),
            _ => Err(HttpError::UnknownVersion),
        }
    }
}

impl From<HttpVersion> for &str {
    fn from(value: HttpVersion) -> Self {
        match value {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
            HttpVersion::Http3 => "HTTP/3",
        }
    }
}

impl TryFrom<&str> for HttpMethod {
    type Error = HttpError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "OPTIONS" => Ok(HttpMethod::Options),
            "GET" => Ok(HttpMethod::Get),
            "HEAD" => Ok(HttpMethod::Head),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "TRACE" => Ok(HttpMethod::Trace),
            "CONNECT" => Ok(HttpMethod::Connect),
            _ => Err(HttpError::UnknownVersion),
        }
    }
}
