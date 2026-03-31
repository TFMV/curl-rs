use crate::http::headers::{HeaderMap, HeaderName, HeaderValue};
use bytes::{Bytes, BytesMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Connect,
    Trace,
    Custom(&'static str),
}

impl Method {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes.to_ascii_uppercase().as_slice() {
            b"GET" => Some(Method::Get),
            b"POST" => Some(Method::Post),
            b"PUT" => Some(Method::Put),
            b"DELETE" => Some(Method::Delete),
            b"HEAD" => Some(Method::Head),
            b"OPTIONS" => Some(Method::Options),
            b"PATCH" => Some(Method::Patch),
            b"CONNECT" => Some(Method::Connect),
            b"TRACE" => Some(Method::Trace),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Patch => "PATCH",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(u16);

impl StatusCode {
    pub fn new(code: u16) -> Option<Self> {
        if (100..600).contains(&code) {
            Some(Self(code))
        } else {
            None
        }
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn as_str(&self) -> &'static str {
        match self.0 {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            409 => "Conflict",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "Unknown",
        }
    }

    pub fn is_informational(&self) -> bool {
        (100..200).contains(&self.0)
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.0)
    }

    pub fn is_redirection(&self) -> bool {
        (300..400).contains(&self.0)
    }

    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.0)
    }

    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.0)
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpVersion(u8, u8);

impl HttpVersion {
    pub const HTTP_1_0: Self = Self(1, 0);
    pub const HTTP_1_1: Self = Self(1, 1);

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"HTTP/1.0" => Some(Self::HTTP_1_0),
            b"HTTP/1.1" => Some(Self::HTTP_1_1),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match (self.0, self.1) {
            (1, 0) => "HTTP/1.0",
            (1, 1) => "HTTP/1.1",
            _ => "HTTP/1.1",
        }
    }
}

impl Default for HttpVersion {
    fn default() -> Self {
        Self::HTTP_1_1
    }
}

pub struct RequestLine<'a> {
    pub method: Method,
    pub uri: &'a str,
    pub version: HttpVersion,
}

pub struct ResponseStart {
    pub version: HttpVersion,
    pub status_code: StatusCode,
    pub reason: Bytes,
}

pub struct ParsedRequest {
    pub method: Method,
    pub uri: Bytes,
    pub version: HttpVersion,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
}

pub struct ParsedResponse {
    pub version: HttpVersion,
    pub status_code: StatusCode,
    pub reason: Bytes,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidMethod,
    InvalidUri,
    InvalidVersion,
    InvalidStatusCode,
    InvalidHeader,
    Incomplete,
    TooLarge,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMethod => write!(f, "invalid HTTP method"),
            Self::InvalidUri => write!(f, "invalid URI"),
            Self::InvalidVersion => write!(f, "invalid HTTP version"),
            Self::InvalidStatusCode => write!(f, "invalid status code"),
            Self::InvalidHeader => write!(f, "invalid header"),
            Self::Incomplete => write!(f, "incomplete HTTP message"),
            Self::TooLarge => write!(f, "HTTP message too large"),
        }
    }
}

impl std::error::Error for ParseError {}

pub struct HttpParser {
    max_header_size: usize,
    #[allow(dead_code)]
    max_body_size: usize,
}

impl Default for HttpParser {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpParser {
    pub fn new() -> Self {
        Self {
            max_header_size: 80 * 1024,
            max_body_size: 64 * 1024 * 1024,
        }
    }

    pub fn with_limits(max_header_size: usize, max_body_size: usize) -> Self {
        Self {
            max_header_size,
            max_body_size,
        }
    }

    pub fn parse_request(&self, data: &[u8]) -> Result<ParsedRequest, ParseError> {
        let mut parser = RequestParser {
            data,
            pos: 0,
            max_header_size: self.max_header_size,
        };
        parser.parse()
    }

    pub fn parse_response(&self, data: &[u8]) -> Result<ParsedResponse, ParseError> {
        let mut parser = ResponseParser {
            data,
            pos: 0,
            max_header_size: self.max_header_size,
        };
        parser.parse()
    }
}

struct RequestParser<'a> {
    data: &'a [u8],
    pos: usize,
    max_header_size: usize,
}

impl<'a> RequestParser<'a> {
    fn parse(&mut self) -> Result<ParsedRequest, ParseError> {
        let line = self.read_line()?;
        let parts: Vec<&[u8]> = line.split(|&b| b == b' ').collect();

        if parts.len() != 3 {
            return Err(ParseError::InvalidUri);
        }

        let method = Method::from_bytes(parts[0]).ok_or(ParseError::InvalidMethod)?;
        let uri = Bytes::copy_from_slice(parts[1]);
        let version = HttpVersion::from_bytes(parts[2]).ok_or(ParseError::InvalidVersion)?;

        let (headers, body) = self.parse_headers_and_body()?;

        Ok(ParsedRequest {
            method,
            uri,
            version,
            headers,
            body,
        })
    }

    fn read_line(&mut self) -> Result<&'a [u8], ParseError> {
        let start = self.pos;
        while self.pos < self.data.len() {
            if self.data[self.pos] == b'\n' {
                let line = &self.data[start..self.pos];
                self.pos += 1;
                if !line.is_empty() && line[line.len() - 1] == b'\r' {
                    return Ok(&line[..line.len() - 1]);
                }
                return Ok(line);
            }
            self.pos += 1;
        }
        Err(ParseError::Incomplete)
    }

    fn parse_headers_and_body(&mut self) -> Result<(HeaderMap, Option<Bytes>), ParseError> {
        let mut headers = HeaderMap::new();

        loop {
            let line = self.read_line()?;
            if line.is_empty() {
                break;
            }

            if headers.len() > 100 || self.pos > self.max_header_size {
                return Err(ParseError::TooLarge);
            }

            let (name, value) = Self::parse_header(line)?;
            headers.append(name, value);
        }

        let body = if self.pos < self.data.len() {
            Some(Bytes::copy_from_slice(&self.data[self.pos..]))
        } else {
            None
        };

        Ok((headers, body))
    }

    fn parse_header(line: &[u8]) -> Result<(HeaderName, HeaderValue), ParseError> {
        let colon_pos = line
            .iter()
            .position(|&b| b == b':')
            .ok_or(ParseError::InvalidHeader)?;

        let name = HeaderName::from_bytes(&line[..colon_pos]).ok_or(ParseError::InvalidHeader)?;

        let mut value_start = colon_pos + 1;
        while value_start < line.len() && line[value_start] == b' ' {
            value_start += 1;
        }

        let mut value_end = line.len();
        while value_end > value_start
            && (line[value_end - 1] == b' ' || line[value_end - 1] == b'\r')
        {
            value_end -= 1;
        }

        let value = HeaderValue::from_bytes(&line[value_start..value_end]);

        Ok((name.to_lowercase(), value))
    }
}

struct ResponseParser<'a> {
    data: &'a [u8],
    pos: usize,
    max_header_size: usize,
}

impl<'a> ResponseParser<'a> {
    fn parse(&mut self) -> Result<ParsedResponse, ParseError> {
        let line = self.read_line()?;
        let parts: Vec<&[u8]> = line.split(|&b| b == b' ').collect();

        if parts.len() < 2 {
            return Err(ParseError::InvalidStatusCode);
        }

        let version = HttpVersion::from_bytes(parts[0]).ok_or(ParseError::InvalidVersion)?;
        let status_code = StatusCode::new(
            std::str::from_utf8(parts[1])
                .ok()
                .and_then(|s| s.parse().ok())
                .ok_or(ParseError::InvalidStatusCode)?,
        )
        .ok_or(ParseError::InvalidStatusCode)?;

        let reason = if parts.len() > 2 {
            let start = if parts[2][0] == b'"' { 1 } else { 0 };
            let end = parts[2].len() - (if parts[2].last() == Some(&b'"') { 1 } else { 0 });
            Bytes::copy_from_slice(&parts[2][start..end])
        } else {
            Bytes::new()
        };

        let (headers, body) = self.parse_headers_and_body()?;

        Ok(ParsedResponse {
            version,
            status_code,
            reason,
            headers,
            body,
        })
    }

    fn read_line(&mut self) -> Result<&'a [u8], ParseError> {
        let start = self.pos;
        while self.pos < self.data.len() {
            if self.data[self.pos] == b'\n' {
                let line = &self.data[start..self.pos];
                self.pos += 1;
                if !line.is_empty() && line[line.len() - 1] == b'\r' {
                    return Ok(&line[..line.len() - 1]);
                }
                return Ok(line);
            }
            self.pos += 1;
        }
        Err(ParseError::Incomplete)
    }

    fn parse_headers_and_body(&mut self) -> Result<(HeaderMap, Option<Bytes>), ParseError> {
        let mut headers = HeaderMap::new();

        loop {
            let line = self.read_line()?;
            if line.is_empty() {
                break;
            }

            if headers.len() > 100 || self.pos > self.max_header_size {
                return Err(ParseError::TooLarge);
            }

            let (name, value) = Self::parse_header(line)?;
            headers.append(name, value);
        }

        let body = if self.pos < self.data.len() {
            Some(Bytes::copy_from_slice(&self.data[self.pos..]))
        } else {
            None
        };

        Ok((headers, body))
    }

    fn parse_header(line: &[u8]) -> Result<(HeaderName, HeaderValue), ParseError> {
        let colon_pos = line
            .iter()
            .position(|&b| b == b':')
            .ok_or(ParseError::InvalidHeader)?;

        let name = HeaderName::from_bytes(&line[..colon_pos]).ok_or(ParseError::InvalidHeader)?;

        let mut value_start = colon_pos + 1;
        while value_start < line.len() && line[value_start] == b' ' {
            value_start += 1;
        }

        let mut value_end = line.len();
        while value_end > value_start
            && (line[value_end - 1] == b' ' || line[value_end - 1] == b'\r')
        {
            value_end -= 1;
        }

        let value = HeaderValue::from_bytes(&line[value_start..value_end]);

        Ok((name.to_lowercase(), value))
    }
}

pub fn build_request(method: Method, uri: &str, headers: &HeaderMap, body: Option<&[u8]>) -> Bytes {
    let mut buf = BytesMut::with_capacity(256);

    buf.extend_from_slice(method.as_str().as_bytes());
    buf.extend_from_slice(b" ");
    buf.extend_from_slice(uri.as_bytes());
    buf.extend_from_slice(b" HTTP/1.1\r\n");

    for (name, value) in headers.iter() {
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(b": ");
        buf.extend_from_slice(value.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    buf.extend_from_slice(b"\r\n");

    if let Some(body) = body {
        buf.extend_from_slice(body);
    }

    buf.freeze()
}

pub fn build_response(
    version: HttpVersion,
    status_code: StatusCode,
    reason: &str,
    headers: &HeaderMap,
    body: Option<&[u8]>,
) -> Bytes {
    let mut buf = BytesMut::with_capacity(256);

    buf.extend_from_slice(version.as_str().as_bytes());
    buf.extend_from_slice(b" ");
    buf.extend_from_slice(status_code.as_u16().to_string().as_bytes());
    buf.extend_from_slice(b" ");
    buf.extend_from_slice(reason.as_bytes());
    buf.extend_from_slice(b"\r\n");

    for (name, value) in headers.iter() {
        buf.extend_from_slice(name.as_bytes());
        buf.extend_from_slice(b": ");
        buf.extend_from_slice(value.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    buf.extend_from_slice(b"\r\n");

    if let Some(body) = body {
        buf.extend_from_slice(body);
    }

    buf.freeze()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_from_bytes() {
        assert_eq!(Method::from_bytes(b"GET"), Some(Method::Get));
        assert_eq!(Method::from_bytes(b"POST"), Some(Method::Post));
        assert_eq!(Method::from_bytes(b"INVALID"), None);
    }

    #[test]
    fn test_status_code() {
        let code = StatusCode::new(200).unwrap();
        assert!(code.is_success());
        assert_eq!(code.as_str(), "OK");

        let code = StatusCode::new(404).unwrap();
        assert!(code.is_client_error());
        assert_eq!(code.as_str(), "Not Found");
    }

    #[test]
    fn test_parse_request() {
        let parser = HttpParser::new();
        let data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

        let req = parser.parse_request(data).unwrap();
        assert_eq!(req.method, Method::Get);
        assert_eq!(req.uri.as_ref(), b"/");
        assert!(req.headers.contains(&HeaderName::from("host")));
    }

    #[test]
    fn test_parse_response() {
        let parser = HttpParser::new();
        let data = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello";

        let res = parser.parse_response(data).unwrap();
        assert_eq!(res.status_code.as_u16(), 200);
        assert!(res.body.is_some());
    }

    #[test]
    fn test_build_request() {
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from("Host"), HeaderValue::from("example.com"));

        let req = build_request(Method::Get, "/", &headers, None);

        assert!(req.starts_with(b"GET / HTTP/1.1"));
        assert!(req.ends_with(b"\r\n\r\n"));
    }
}
