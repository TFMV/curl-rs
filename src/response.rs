use crate::http::{HeaderMap, HeaderName, HeaderValue, HttpVersion, StatusCode};
use bytes::Bytes;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Response {
    pub version: HttpVersion,
    pub status_code: StatusCode,
    pub reason: String,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub url: Option<String>,
    pub redirected: bool,
    pub redirect_count: usize,
}

impl Response {
    pub fn new() -> Self {
        Self {
            version: HttpVersion::default(),
            status_code: StatusCode::new(200).unwrap(),
            reason: String::new(),
            headers: HeaderMap::new(),
            body: None,
            url: None,
            redirected: false,
            redirect_count: 0,
        }
    }

    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    pub fn status(&self) -> u16 {
        self.status_code.as_u16()
    }

    pub fn status_text(&self) -> &str {
        self.status_code.as_str()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body(&self) -> Option<&Bytes> {
        self.body.as_ref()
    }

    pub fn body_str(&self) -> Option<&str> {
        self.body.as_ref().and_then(|b| std::str::from_utf8(b).ok())
    }

    pub fn content_length(&self) -> Option<u64> {
        self.headers
            .get(&HeaderName::from("content-length"))
            .and_then(|v| v.as_str().parse().ok())
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers
            .get(&HeaderName::from("content-type"))
            .map(|v| v.as_str())
    }

    pub fn charset(&self) -> Option<&str> {
        self.content_type()
            .and_then(|ct| ct.split(';').nth(1))
            .and_then(|c| c.trim().strip_prefix("charset="))
    }

    pub fn is_success(&self) -> bool {
        self.status_code.is_success()
    }

    pub fn is_redirection(&self) -> bool {
        self.status_code.is_redirection()
    }

    pub fn is_client_error(&self) -> bool {
        self.status_code.is_client_error()
    }

    pub fn is_server_error(&self) -> bool {
        self.status_code.is_server_error()
    }

    pub fn location(&self) -> Option<&str> {
        self.headers
            .get(&HeaderName::from("location"))
            .map(|v| v.as_str())
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn text(&self) -> String {
        self.body_str().unwrap_or("").to_string()
    }

    #[cfg(feature = "json")]
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body.as_deref().unwrap_or(&Bytes::new()))
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.version.as_str(),
            self.status_code.as_u16(),
            self.reason
        )
    }
}

pub struct ResponseBuilder {
    response: Response,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            response: Response::new(),
        }
    }

    pub fn version(mut self, version: HttpVersion) -> Self {
        self.response.version = version;
        self
    }

    pub fn status_code(mut self, code: StatusCode) -> Self {
        self.response.status_code = code;
        self
    }

    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.response.reason = reason.into();
        self
    }

    pub fn header(mut self, name: impl Into<HeaderName>, value: impl Into<HeaderValue>) -> Self {
        self.response.headers.insert(name.into(), value.into());
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.response.headers = headers;
        self
    }

    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.response.body = Some(body.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.response.url = Some(url.into());
        self
    }

    pub fn redirected(mut self, redirected: bool) -> Self {
        self.response.redirected = redirected;
        self
    }

    pub fn redirect_count(mut self, count: usize) -> Self {
        self.response.redirect_count = count;
        self
    }

    pub fn build(self) -> Response {
        self.response
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<crate::http::parser::ParsedResponse> for Response {
    fn from(parsed: crate::http::parser::ParsedResponse) -> Self {
        Self {
            version: parsed.version,
            status_code: parsed.status_code,
            reason: String::from_utf8_lossy(&parsed.reason).to_string(),
            headers: parsed.headers,
            body: parsed.body,
            url: None,
            redirected: false,
            redirect_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_status() {
        let response = Response::builder()
            .status_code(StatusCode::new(200).unwrap())
            .reason("OK")
            .build();

        assert!(response.is_success());
        assert_eq!(response.status(), 200);
    }

    #[test]
    fn test_response_body_str() {
        let response = Response::builder().body("Hello, World!").build();

        assert_eq!(response.body_str(), Some("Hello, World!"));
    }

    #[test]
    fn test_response_location() {
        let mut response = Response::new();
        response.headers.insert(
            HeaderName::from("location"),
            HeaderValue::from("https://example.com/redirect"),
        );

        assert_eq!(response.location(), Some("https://example.com/redirect"));
    }
}
