use std::time::Duration;

pub mod tcp;
pub mod tls;

pub use tcp::TcpStream;
pub use tls::TlsStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    pub fn from_scheme(scheme: &str) -> Option<Self> {
        match scheme.to_lowercase().as_str() {
            "http" => Some(Protocol::Http),
            "https" => Some(Protocol::Https),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Timeout(pub Option<Duration>);

impl Timeout {
    pub fn from_secs(secs: u64) -> Self {
        Self(Some(Duration::from_secs(secs)))
    }

    pub fn none() -> Self {
        Self(None)
    }
}
