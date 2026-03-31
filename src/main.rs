use curl_rs::cli::Cli;
use curl_rs::{Client, Method, Request};
use std::io::{self, Write};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.url.is_empty() {
        eprintln!("curl-rs: missing URL");
        std::process::exit(2);
    }

    let client = Client::new()
        .follow_redirects(cli.location)
        .max_redirects(cli.max_redirs.unwrap_or(10))
        .verify_ssl(!cli.insecure);

    let client = if let Some(timeout) = cli.max_time {
        client.timeout(std::time::Duration::from_secs_f64(timeout))
    } else {
        client
    };

    let client = if let Some(timeout) = cli.connect_timeout() {
        client.connect_timeout(timeout)
    } else {
        client
    };

    let mut request = if let Some(method) = cli.get_method() {
        let method = match method.to_uppercase().as_str() {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            "PATCH" => Method::Patch,
            "OPTIONS" => Method::Options,
            _ => Method::Custom(Box::leak(method.into_boxed_str())),
        };
        Request::new(method, &cli.url)
    } else {
        Request::get(&cli.url)
    };

    request = request.user_agent("curl-rs/0.1.0");

    for (name, value) in cli.to_headers() {
        request = request.header(name, value);
    }

    if let Some(ref data) = cli.data {
        request = request.body(data.clone().into_bytes());
        if cli.get_method().is_none() {
            request = request.header("Content-Type", "application/x-www-form-urlencoded");
        }
    }

    let response = client.execute(request).await;

    match response {
        Ok(resp) => {
            if cli.include {
                println!("HTTP/1.1 {} {}", resp.status(), resp.status_text());
                for (name, value) in resp.headers().iter() {
                    println!("{}: {}", name, value);
                }
                println!();
            }

            if !cli.silent {
                if let Some(body) = resp.body() {
                    if cli.output.is_some() || cli.remote_name {
                        // Handle file output
                    } else {
                        let _ = io::stdout().write_all(body);
                    }
                }
            }

            if cli.show_error && !resp.is_success() {
                eprintln!("curl-rs: HTTP request failed with status {}", resp.status());
            }

            if cli.fail && !resp.is_success() {
                std::process::exit(22);
            }

            std::process::exit(0);
        }
        Err(e) => {
            if cli.fail || cli.show_error {
                eprintln!("curl-rs: {}", e);
            }
            std::process::exit(2);
        }
    }
}