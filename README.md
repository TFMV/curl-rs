# curl-rs

A lightweight async HTTP client written in Rust, using only tokio.

## Dependencies

- **tokio** - async runtime (net, io-util, time, rt, macros)
- **bytes** - efficient byte handling

Optional (via `json` feature):
- **serde** / **serde_json** - for `.json()` methods

## Installation

```bash
cargo build --release
./target/release/curl-rs https://example.com
```

## Library Usage

```rust
use curl_rs::{Client, Request, Method};

#[tokio::main]
async fn main() -> Result<(), curl_rs::Error> {
    // Simple GET
    let response = Client::new()
        .execute(Request::get("https://example.com"))
        .await?;

    println!("Status: {}", response.status());
    println!("Body: {}", response.text());

    // POST with headers
    let response = Client::new()
        .timeout(std::time::Duration::from_secs(30))
        .execute(
            Request::post("https://api.example.com/data")
                .header("Authorization", "Bearer token")
                .header("Content-Type", "application/json")
                .body(r#"{"key": "value"}"#)
        )
        .await?;

    Ok(())
}
```

### Convenience Functions

```rust
// Quick GET
let res = curl_rs::get("http://example.com").await?;

// Quick POST
let res = curl_rs::post("http://example.com", "body data").await?;
```

### API Reference

#### Client

```rust
let client = Client::new()
    .timeout(std::time::Duration::from_secs(30))
    .connect_timeout(std::time::Duration::from_secs(5))
    .follow_redirects(true)
    .max_redirects(10)
    .verify_ssl(true)
    .default_header("User-Agent", "my-app/1.0");
```

#### Request

```rust
let request = Request::new(Method::Get, "https://example.com");

// Shorthand constructors
Request::get(url);
Request::post(url);
Request::put(url);
Request::delete(url);

// Builder
Request::post("https://api.example.com")
    .header("Authorization", "Bearer token")
    .header("Content-Type", "application/json")
    .body("request body")
    .timeout(std::time::Duration::from_secs(10))
    .follow_redirects(false);
```

#### Response

```rust
response.status();           // u16 (e.g., 200)
response.status_text();      // &str (e.g., "OK")
response.headers();          // &HeaderMap
response.body();             // Option<&Bytes>
response.text();             // String
response.json::<T>();        // Deserialize JSON (requires json feature)
response.is_success();       // bool (2xx)
response.is_redirection();   // bool (3xx)
response.is_client_error();  // bool (4xx)
response.is_server_error();  // bool (5xx)
response.location();         // Option<&str> for redirects
```

#### Headers

```rust
use curl_rs::{HeaderMap, HeaderName, HeaderValue};

let mut headers = HeaderMap::new();
headers.insert(HeaderName::from("Content-Type"), HeaderValue::from("application/json"));
headers.append(HeaderName::from("Accept"), HeaderValue::from("text/html")); //追加
```

## CLI Usage

```bash
# Simple GET
curl-rs https://example.com

# POST with data
curl-rs -X POST -d "key=value" https://example.com

# Custom headers
curl-rs -H "Authorization: Bearer token" https://api.example.com

# Follow redirects
curl-rs -L https://example.com/redirect

# Timeout (seconds)
curl-rs --max-time 10 https://example.com

# Silent mode
curl-rs -s https://example.com
```

### CLI Options

| Flag | Description |
|------|-------------|
| `-A, --user-agent` | Set User-Agent |
| `-H, --header` | Add custom header |
| `-X, --request` | HTTP method |
| `-d, --data` | POST data |
| `-L, --location` | Follow redirects |
| `--max-time` | Timeout in seconds |
| `-s, --silent` | Silent mode |
| `-i, --include` | Include response headers |
| `-I, --head` | HEAD request |
| `-k, --insecure` | Skip SSL verification |
| `-o, --output` | Write to file |
| `-f, --fail` | Exit on HTTP error |

## Features

- HTTP/1.1 client
- GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH methods
- Custom headers with case-insensitive lookup
- Automatic redirect following
- Request/response body handling
- Connection and request timeouts

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with Clippy
cargo clippy

# Format code
cargo fmt

# Run benchmarks
cargo bench
```

## License

MIT License