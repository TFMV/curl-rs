pub mod cli;
pub mod error;
pub mod http;
pub mod net;
pub mod request;
pub mod response;
pub mod url;

pub use error::{Error, Result};
pub use http::{HeaderMap, HeaderName, HeaderValue, HttpParser, HttpVersion, Method, StatusCode};
pub use request::{Request, RequestBuilder};
pub use response::{Response, ResponseBuilder};

use crate::url::Url;
use std::net::ToSocketAddrs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Client {
    timeout: Option<std::time::Duration>,
    connect_timeout: Option<std::time::Duration>,
    follow_redirects: bool,
    max_redirects: usize,
    verify_ssl: bool,
    default_headers: HeaderMap,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            timeout: None,
            connect_timeout: None,
            follow_redirects: true,
            max_redirects: 10,
            verify_ssl: true,
            default_headers: HeaderMap::new(),
        }
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn connect_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.connect_timeout = Some(timeout);
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

    pub fn default_header(
        mut self,
        name: impl Into<HeaderName>,
        value: impl Into<HeaderValue>,
    ) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    pub async fn execute(&self, mut request: Request) -> Result<Response> {
        let mut redirect_count = 0;

        loop {
            let url = Url::parse(&request.uri).map_err(|e| Error::InvalidUrl(e.to_string()))?;

            let host = url.host_str();
            let port = url.port().unwrap_or(url.default_port());
            let scheme = url.scheme();

            let addr = format!("{}:{}", host, port)
                .to_socket_addrs()
                .map_err(|e| Error::Connection(e.to_string()))?
                .next()
                .ok_or_else(|| Error::Connection("failed to resolve host".to_string()))?;

            let _use_tls = scheme == "https";

            let mut tcp = tokio::net::TcpStream::connect(addr)
                .await
                .map_err(|e| Error::Connection(e.to_string()))?;

            let (mut read, mut write) = tcp.split();

            let mut headers = self.default_headers.clone();
            headers.extend(request.headers.clone());

            if !headers.contains(&HeaderName::from("host")) {
                headers.insert(HeaderName::from("host"), HeaderValue::from(host));
            }

            if !headers.contains(&HeaderName::from("connection")) {
                headers.insert(HeaderName::from("connection"), HeaderValue::from("close"));
            }

            let path = url.path();
            let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
            let uri = if path.is_empty() {
                "/".to_string()
            } else {
                path.to_string()
            } + &query;

            let request_line = format!("{} {} HTTP/1.1\r\n", request.method.as_str(), uri);

            let mut request_bytes = request_line.as_bytes().to_vec();

            for (name, value) in headers.iter() {
                request_bytes.extend_from_slice(name.as_bytes());
                request_bytes.extend_from_slice(b": ");
                request_bytes.extend_from_slice(value.as_bytes());
                request_bytes.extend_from_slice(b"\r\n");
            }

            request_bytes.extend_from_slice(b"\r\n");

            if let Some(body) = &request.body {
                request_bytes.extend_from_slice(body);
            }

            write
                .write_all(&request_bytes)
                .await
                .map_err(|e| Error::Io(e.to_string()))?;
            write
                .flush()
                .await
                .map_err(|e| Error::Io(e.to_string()))?;

            let mut response_bytes = Vec::new();
            read.read_to_end(&mut response_bytes)
                .await
                .map_err(|e| Error::Io(e.to_string()))?;

            let parser = HttpParser::new();
            let parsed = parser
                .parse_response(&response_bytes)
                .map_err(|e| Error::HttpParse(e.to_string()))?;

            let mut response = Response::from(parsed);
            response.url = Some(request.uri.clone());

            if self.follow_redirects
                && response.is_redirection()
                && redirect_count < self.max_redirects
            {
                if let Some(location) = response.location() {
                    if let Ok(base) = Url::parse(&request.uri) {
                        if let Ok(next_url) = base.join(location) {
                            request.uri = next_url.to_string();
                            redirect_count += 1;
                            continue;
                        }
                    }
                }
            }

            return Ok(response);
        }
    }
}

pub async fn get(url: &str) -> Result<Response> {
    Client::new()
        .execute(Request::get(url))
        .await
}

pub async fn post(url: &str, body: impl Into<bytes::Bytes>) -> Result<Response> {
    Client::new()
        .execute(Request::post(url).body(body))
        .await
}

pub use bytes::Bytes;