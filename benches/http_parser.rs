use curl_rs::http::{HttpParser, Method};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    let parser = HttpParser::new();

    c.bench_function("parse_simple_request", |b| {
        let data =
            b"GET / HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl-rs\r\nAccept: */*\r\n\r\n";
        b.iter(|| parser.parse_request(data));
    });

    c.bench_function("parse_request_with_body", |b| {
        let data = b"POST /submit HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"key\":\"value\"}";
        b.iter(|| parser.parse_request(data));
    });

    c.bench_function("parse_simple_response", |b| {
        let data = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\n\r\nHello, World!";
        b.iter(|| parser.parse_response(data));
    });

    c.bench_function("parse_response_many_headers", |b| {
        let data = b"HTTP/1.1 200 OK\r\nServer: nginx\r\nDate: Mon, 01 Jan 2024 00:00:00 GMT\r\nContent-Type: text/html\r\nContent-Length: 1234\r\nConnection: keep-alive\r\n\r\n";
        b.iter(|| parser.parse_response(data));
    });
}

criterion::criterion_group!(benches, criterion_benchmark);
criterion::criterion_main!(benches);
