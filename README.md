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

This is the [tokio](https://tokio.rs) branch for fix-rs. It's used to evaluate when/if tokio should be supported. The goal is to have an API compatible with `fixt::engine::Engine` but have the backend using tokio instead of mio. Then, ideally the tokio parts could be reused for custom engines with different performance requirements. This new version is found in the `src/engine/` directory.

Thoughts so far as of tokio-core 0.1.6:

- Cryptic compiler errors make it very hard to use.
- Sharing state between futures is complicated and makes reusability difficult.
- Performance is slightly worse than using mio directly.

## Examples

- **Client**: [examples/client.rs](examples/client.rs) shows how to initiate a connection and communicate with a FIX engine.
- **Server**: [examples/server.rs](examples/server.rs) shows how to accept connections and function as a FIX engine.

## License

fix-rs is dual licensed under both the MIT license and the Apache License (Version 2.0). Pick the license that is more convenient.

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
