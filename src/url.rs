use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
    query: Option<String>,
}

#[derive(Debug)]
pub enum UrlError {
    InvalidUrl(String),
    MissingHost,
    InvalidPort,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl(s) => write!(f, "invalid URL: {}", s),
            Self::MissingHost => write!(f, "missing host"),
            Self::InvalidPort => write!(f, "invalid port"),
        }
    }
}

impl std::error::Error for UrlError {}

impl Url {
    pub fn parse(s: &str) -> Result<Self, UrlError> {
        let (scheme, rest) = s
            .split_once("://")
            .ok_or_else(|| UrlError::InvalidUrl(s.to_string()))?;

        let scheme = scheme.to_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(UrlError::InvalidUrl(format!("unknown scheme: {}", scheme)));
        }

        let (host_port, path_query) = rest.split_once('/').unwrap_or((rest, ""));

        let (host, port) = if let Some((h, p)) = host_port.rsplit_once(':') {
            let port: u16 = p.parse().map_err(|_| UrlError::InvalidPort)?;
            (h.to_string(), Some(port))
        } else {
            (host_port.to_string(), None)
        };

        if host.is_empty() {
            return Err(UrlError::MissingHost);
        }

        let (path, query) = if let Some((p, q)) = path_query.split_once('?') {
            let path = if p.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", p)
            };
            (path, Some(q.to_string()))
        } else {
            let path = if path_query.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", path_query)
            };
            (path, None)
        };

        Ok(Self {
            scheme,
            host,
            port,
            path,
            query,
        })
    }

    pub fn join(&self, relative: &str) -> Result<Self, UrlError> {
        if relative.starts_with("http://") || relative.starts_with("https://") {
            return Self::parse(relative);
        }

        let base_path = self.path.trim_end_matches('/');

        if relative.starts_with('/') {
            return Ok(Self {
                scheme: self.scheme.clone(),
                host: self.host.clone(),
                port: self.port,
                path: relative.to_string(),
                query: None,
            });
        }

        if let Some((path, query)) = relative.split_once('?') {
            let new_path = if path.is_empty() {
                base_path.to_string()
            } else {
                format!("{}/{}", base_path, path)
            };
            return Ok(Self {
                scheme: self.scheme.clone(),
                host: self.host.clone(),
                port: self.port,
                path: new_path,
                query: Some(query.to_string()),
            });
        }

        let new_path = if relative.is_empty() {
            base_path.to_string()
        } else {
            format!("{}/{}", base_path, relative)
        };

        Ok(Self {
            scheme: self.scheme.clone(),
            host: self.host.clone(),
            port: self.port,
            path: new_path,
            query: None,
        })
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host_str(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn default_port(&self) -> u16 {
        match self.scheme.as_str() {
            "https" => 443,
            _ => 80,
        }
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.scheme, self.host)?;
        if let Some(port) = self.port {
            let default = self.default_port();
            if port != default {
                write!(f, ":{}", port)?;
            }
        }
        write!(f, "{}", self.path)?;
        if let Some(ref q) = self.query {
            write!(f, "?{}", q)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http() {
        let url = Url::parse("http://example.com").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), "example.com");
        assert_eq!(url.port(), None);
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_parse_https_with_port() {
        let url = Url::parse("https://example.com:8443/path").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), "example.com");
        assert_eq!(url.port(), Some(8443));
        assert_eq!(url.path(), "/path");
    }

    #[test]
    fn test_parse_with_query() {
        let url = Url::parse("http://example.com/api?key=value").unwrap();
        assert_eq!(url.path(), "/api");
        assert_eq!(url.query(), Some("key=value"));
    }

    #[test]
    fn test_join_relative() {
        let url = Url::parse("http://example.com/path").unwrap();
        let joined = url.join("next").unwrap();
        assert_eq!(joined.path(), "/path/next");
    }

    #[test]
    fn test_join_absolute() {
        let url = Url::parse("http://example.com/path").unwrap();
        let joined = url.join("/absolute").unwrap();
        assert_eq!(joined.path(), "/absolute");
    }
}
