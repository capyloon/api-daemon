# (Integration) Test server

[![Build Status](https://github.com/ChriFo/test-server-rs/workflows/Test/badge.svg)](https://github.com/ChriFo/test-server-rs/actions)
[![codecov](https://codecov.io/gh/ChriFo/test-server-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/ChriFo/test-server-rs)

## Usage

```toml
[dev-dependencies]
test-server = { git = "https://github.com/ChriFo/test-server-rs", tag = "0.9.1" }
```

[HttpResponse](https://actix.rs/api/actix-web/stable/actix_web/struct.HttpResponse.html) and [HttpRequest](https://actix.rs/api/actix-web/stable/actix_web/struct.HttpRequest.html) are re-exports from [actix-web](https://github.com/actix/actix-web).

```rust,skt-test
// start server at random port
let _ = test_server::new("127.0.0.1:0", test_server::HttpResponse::Ok)?;

// start server at given port
let server = test_server::new("127.0.0.1:8080", |req: test_server::HttpRequest| {
    println!("{:#?}", req);
    test_server::HttpResponse::Ok().body("hello world")
})?;

// request against server
let _ = get_request(&server.url());

assert_eq!(1, server.requests.len());

// requests are Request from crate http (which is re-exported as http as well)
let last_request = server.requests.next().unwrap();

assert_eq!("GET", last_request.method());
assert_eq!("/", last_request.uri().path());
// body, headers and query params are also available
```

For more examples have a look at the [tests](https://github.com/ChriFo/test-server-rs/blob/master/tests/server.rs).
