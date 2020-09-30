[![build status](https://git.kaiostech.com/KaiOS/sidl/badges/master/build.svg)](https://git.kaiostech.com/KaiOS/sidl/commits/master)

This repository is organized as follows:
- [parser](./parser): the SIDL parser, producing an AST that is reused by other crates.
- [codegen](./codegen): code generator for protocol buffer and Rust.
- [docs](./docs): documentation of the sidl syntax.
- [common](./common): common Rust code shared by services.
- [daemon](./daemon): the web socket daemon.
- [tcpsocket-service](./tcpsocket-service): a service exposing access to tcp socket endpoints.
- [libsignal-sys](./libsignal-sys): FFI bindings to libsignal-protocol-c.
- [libsignal-service](./libsignal-service): a service exposing api used for the Signal protocol.
- [prebuilts](./prebuilts): pre-compiled daemon for ARMv7, and other generated files for clients.
