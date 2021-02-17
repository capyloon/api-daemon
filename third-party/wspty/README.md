# wspty - xterm.js example with Rust Tokio and pty backend server

# Usage

### Start the websocket backend server

```
cargo run --example server
```
### Open web page

```
./assets/index.html
```

# Related Projects

* The wire protocol follows https://github.com/freman/goterm.

* Pty and tokio integration is inspired by [tokio-pty-process](https://crates.io/crates/tokio-pty-process).

* This project is using an old version of [xterm.js](https://xtermjs.org/). To use the latest verion, javascript code in the index.html should change accordingly.

# License

This project is licensed under the MIT license.