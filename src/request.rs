use crate::http::{HeaderMap, HeaderName, HeaderValue, HttpVersion, Method};
use bytes::Bytes;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub version: HttpVersion,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub timeout: Option<std::time::Duration>,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub verify_ssl: bool,
}

impl Request {
    pub fn new(method: Method, uri: impl Into<String>) -> Self {
        Self {
            method,
            uri: uri.into(),
            version: HttpVersion::default(),
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
            max_redirects: 10,
            verify_ssl: true,
        }
    }

    pub fn get(uri: impl Into<String>) -> Self {
        Self::new(Method::Get, uri)
    }

    pub fn post(uri: impl Into<String>) -> Self {
        Self::new(Method::Post, uri)
    }

    pub fn put(uri: impl Into<String>) -> Self {
        Self::new(Method::Put, uri)
    }

    pub fn delete(uri: impl Into<String>) -> Self {
        Self::new(Method::Delete, uri)
    }

    pub fn head(uri: impl Into<String>) -> Self {
        Self::new(Method::Head, uri)
    }

    pub fn header(mut self, name: impl Into<HeaderName>, value: impl Into<HeaderValue>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn headers(mut self, headers: impl Into<HeaderMap>) -> Self {
        self.headers = headers.into();
        self
    }

    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Some(body.into());
        if !self.headers.contains(&HeaderName::from("content-length")) {
            if let Some(body) = &self.body {
                self.headers.insert(
                    HeaderName::from("Content-Length"),
                    HeaderValue::from(body.len().to_string()),
                );
            }
        }
        self
    }

    pub fn body_str(mut self, body: impl Into<String>) -> Self {
        let bytes = Bytes::from(body.into());
        self = self.body(bytes.clone());
        if !self.headers.contains(&HeaderName::from("content-type")) {
            self.headers.insert(
                HeaderName::from("Content-Type"),
                HeaderValue::from("application/x-www-form-urlencoded"),
            );
        }
        self
    }

    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(value)?;
        let len = body.len();
        self.body = Some(Bytes::from(body));
        self.headers.insert(
            HeaderName::from("Content-Type"),
            HeaderValue::from("application/json"),
        );
        self.headers.insert(
            HeaderName::from("Content-Length"),
            HeaderValue::from(len.to_string()),
        );
        Ok(self)
    }

    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn follow_redirects(mut self, enabled: bool) -> Self {
        self.follow_redirects = enabled;
        self
    }

    pub fn max_redirects(mut self, count: usize) -> Self {
        self.max_redirects = count;
        self
    }

    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.headers.insert(
            HeaderName::from("User-Agent"),
            HeaderValue::from(user_agent),
        );
        self
    }

    pub fn accept(mut self, accept: &str) -> Self {
        self.headers
            .insert(HeaderName::from("Accept"), HeaderValue::from(accept));
        self
    }

    pub fn build(&self) -> Bytes {
        use crate::http::parser::build_request as http_build_request;
        let body = self.body.as_deref();
        http_build_request(self.method, &self.uri, &self.headers, body)
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn set_uri(&mut self, uri: String) {
        self.uri = uri;
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new(Method::Get, "/")
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.method, self.uri, self.version.as_str())
    }
}

pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    pub fn new(method: Method, uri: impl Into<String>) -> Self {
        Self {
            request: Request::new(method, uri),
        }
    }

    pub fn get(uri: impl Into<String>) -> Self {
        Self::new(Method::Get, uri)
    }

    pub fn post(uri: impl Into<String>) -> Self {
        Self::new(Method::Post, uri)
    }

    pub fn put(uri: impl Into<String>) -> Self {
        Self::new(Method::Put, uri)
    }

    pub fn delete(uri: impl Into<String>) -> Self {
        Self::new(Method::Delete, uri)
    }

    pub fn header(mut self, name: impl Into<HeaderName>, value: impl Into<HeaderValue>) -> Self {
        self.request.headers.insert(name.into(), value.into());
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.request.headers = headers;
        self
    }

    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.request.body = Some(body.into());
        self
    }

    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.request.timeout = Some(duration);
        self
    }

    pub fn build(self) -> Request {
        self.request
    }
}
