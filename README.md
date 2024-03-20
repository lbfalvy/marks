# URL Marks

A lightning fast bookmark manager fit to be your browser home page

## Development

Working with Webassembly in Rust requires some preparation:

```
$ rustup target add wasm32-unknown-unknown
$ cargo install trunk
``

Make sure also that `trunk` is in `$PATH`. By default, cargo install places programs in `~/.cargo/bin`.

The "run" xtask (invokable as `cargo xtask run`) starts the server and the client. The client is hot reloaded by `trunk` automatically, but hot reloading has not yet been written for the server. Help in this regard is appreciated.

Diesel-cli is exposed as the `diesel` xtask. This is useful because unlike Cargo commands or the default diesel behaviour, the xtask runner can switch to the appropriate directory first so the commands are available anywhere in the project.