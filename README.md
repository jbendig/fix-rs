# fix-rs

[![Tokei SLoC Count](https://tokei.rs/b1/github/jbendig/fix-rs)](https://github.com/jbendig/fix-rs)

**fix-rs** is a [FIX](http://www.fixtradingcommunity.org/) (Financial Information Exchange) engine written in [Rust](https://www.rust-lang.org/). It supports the following FIX versions:

- FIX 4.0
- FIX 4.1
- FIX 4.2
- FIX 4.3
- FIX 4.4
- FIX 5.0
- FIX 5.0 SP1
- FIX 5.0 SP2

## Status

This project is an early work in progress and still under heavy development.

Current progress is focused on evaluating whether to support [tokio](https://tokio.rs).

## Examples

- **Client**: [examples/client.rs](examples/client.rs) shows how to initiate a connection and communicate with a FIX engine.
- **Server**: [examples/server.rs](examples/server.rs) shows how to accept connections and function as a FIX engine.

## License

fix-rs is dual licensed under both the MIT license and the Apache License (Version 2.0). Pick the license that is more convenient.

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
