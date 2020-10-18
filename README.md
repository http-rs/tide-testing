# tide-testing
## a convenient bridge between surf and tide to generate synthetic requests for testing

* [CI ![CI][ci-badge]][ci]
* [API Docs][docs] [![docs.rs docs][docs-badge]][docs]
* [Releases][releases] [![crates.io version][version-badge]][lib-rs]
* [Contributing][contributing]

[ci]: https://github.com/jbr/tide-testing/actions?query=workflow%3ACI
[ci-badge]: https://github.com/jbr/tide-testing/workflows/CI/badge.svg
[releases]: https://github.com/jbr/tide-testing/releases
[docs]: https://docs.rs/tide-testing
[contributing]: https://github.com/jbr/tide-testing/blob/main/.github/CONTRIBUTING.md
[lib-rs]: https://lib.rs/tide-testing
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square
[version-badge]: https://img.shields.io/crates/v/tide-testing.svg?style=flat-square

## Installation
```
$ cargo add -D tide-testing
```

## Example usage:

```rust
let mut app = tide::new();
app.at("/").get(|_| async { Ok("hello!") });

use tide_testing::TideTestingExt;
assert_eq!(app.get("/").recv_string().await?, "hello!");
assert_eq!(
    app.post("/missing").await?.status(),
    tide::http::StatusCode::NotFound
);
```

## Cargo Features:

## Safety
This crate uses ``#![deny(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
