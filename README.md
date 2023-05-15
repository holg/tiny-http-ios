# A tiny-http server, writen in Rust, intended to be used inside an iOS Project

## It is convenient to have some embedded web server inside an iOS project:

+ To serve static files, like html, css, js, images, etc.
+ More importantly to serve WASM files, which are compiled from i.e. Rust code, and then run inside a webview in iOS.

```rust

* [Installation](#installation)