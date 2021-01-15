```rust,skt-test
extern crate anyhow;
extern crate test_server;
extern crate ureq;

fn get_request(url: &str) -> ureq::Response {{
    ureq::get(url).call()
}}

fn main() -> anyhow::Result<()> {{

    {}

    Ok(())
}}
```