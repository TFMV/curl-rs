use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderMap(HashMap<HeaderName, HeaderValue>);

impl Default for HeaderMap {
    fn default() -> Self {
        Self::new()
    }
}

impl HeaderMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn insert(&mut self, name: HeaderName, value: HeaderValue) -> Option<HeaderValue> {
        self.0.insert(name, value)
    }

    pub fn append(&mut self, name: HeaderName, value: HeaderValue) {
        if let Some(existing) = self.0.get_mut(&name) {
            existing.append(value);
        } else {
            self.0.insert(name, value);
        }
    }

    pub fn get(&self, name: &HeaderName) -> Option<&HeaderValue> {
        self.0.get(name)
    }

    pub fn remove(&mut self, name: &HeaderName) -> Option<HeaderValue> {
        self.0.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, name: &HeaderName) -> bool {
        self.0.contains_key(name)
    }

    pub fn extend(&mut self, other: HeaderMap) {
        for (name, value) in other.0 {
            self.append(name, value);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeaderName(Vec<u8>);

impl HeaderName {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        for &b in bytes {
            if !Self::is_valid_header_byte(b) {
                return None;
            }
        }
        Some(Self(bytes.to_vec()))
    }

    fn is_valid_header_byte(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.'
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("")
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_lowercase(&self) -> Self {
        Self(self.0.iter().map(|b| b.to_ascii_lowercase()).collect())
    }
}

impl From<&str> for HeaderName {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for HeaderName {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl std::fmt::Display for HeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderValue(Vec<u8>);

impl HeaderValue {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("")
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn append(&mut self, other: HeaderValue) {
        self.0.push(b',');
        self.0.extend_from_slice(&other.0);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<&str> for HeaderValue {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for HeaderValue {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl std::fmt::Display for HeaderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_name_from_bytes() {
        let name = HeaderName::from_bytes(b"Content-Type").unwrap();
        assert_eq!(name.as_str(), "Content-Type");
    }

    #[test]
    fn test_header_value() {
        let value = HeaderValue::from_str("application/json");
        assert_eq!(value.as_str(), "application/json");
    }

    #[test]
    fn test_header_map_insert_get() {
        let mut map = HeaderMap::new();
        map.insert(
            HeaderName::from("Content-Type"),
            HeaderValue::from("application/json"),
        );

        assert_eq!(
            map.get(&HeaderName::from("Content-Type")).unwrap().as_str(),
            "application/json"
        );
    }

    #[test]
    fn test_header_map_append() {
        let mut map = HeaderMap::new();
        map.insert(HeaderName::from("Accept"), HeaderValue::from("text/html"));
        map.append(
            HeaderName::from("Accept"),
            HeaderValue::from("application/json"),
        );

        assert_eq!(
            map.get(&HeaderName::from("Accept")).unwrap().as_str(),
            "text/html,application/json"
        );
    }
}
