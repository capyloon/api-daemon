# did

**A parser for Decentralized Identifiers (DIDs)**

---

```rust
use did_url::DID;

let did = DID::parse("did:example:alice")?;

// Prints Method = example
println!("Method = {}", did.method());

// Prints Method Id = alice
println!("Method Id = {}", did.method_id());

// Prints DID = did:example:alice
println!("DID = {}", did);

// Prints Joined = did:example:alice?query=true#key-1
println!("Joined = {}", did.join("#key-1")?.join("?query=true")?);
```

## References

- [DID Syntax](https://www.w3.org/TR/did-core/#did-syntax)
- [DID Url Syntax](https://www.w3.org/TR/did-core/#did-url-syntax)
- [DID Parameters](https://www.w3.org/TR/did-core/#did-parameters)
- [Path](https://www.w3.org/TR/did-core/#path)
- [Query](https://www.w3.org/TR/did-core/#query)
- [Fragment](https://www.w3.org/TR/did-core/#fragment)
- [Relative DID Urls](https://www.w3.org/TR/did-core/#relative-did-urls)

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
