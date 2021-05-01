# Running the daemon on desktop.

It's possible to run the daemon on a desktop/laptop without making any changes to the device code:

- Make sure that you have a [stable Rust toolchain](https://rustup.rs) installed for you host architecture.
- Go into the `daemon` directory and run `cargo build --release`.
- Create a reverse port mapping to redirect requests from the device to the host: `adb reverse tcp:8081 tcp:8081`.
- Run the daemon with `RUST_LOG=info cargo run --release`.
