# Changes

## Unreleased - 2022-xx-xx


## 4.2.0 - 2023-01-21
- Minimum supported Rust version (MSRV) is now 1.57 due to transitive `time` dependency.


## 4.1.0 - 2022-03-02
- Add support for `actix` version `0.13`. [#2675]

[#2675]: https://github.com/actix/actix-web/pull/2675


## 4.0.0 - 2022-02-25
- No significant changes since `4.0.0-beta.12`.


## 4.0.0-beta.12 - 2022-02-16
- No significant changes since `4.0.0-beta.11`.


## 4.0.0-beta.11 - 2022-01-31
- No significant changes since `4.0.0-beta.10`.


## 4.0.0-beta.10 - 2022-01-04
- Minimum supported Rust version (MSRV) is now 1.54.


## 4.0.0-beta.9 - 2021-12-27
- No significant changes since `4.0.0-beta.8`.


## 4.0.0-beta.8 - 2021-12-11
- Add `ws:WsResponseBuilder` for building WebSocket session response. [#1920]
- Deprecate `ws::{start_with_addr, start_with_protocols}`. [#1920]
- Minimum supported Rust version (MSRV) is now 1.52.

[#1920]: https://github.com/actix/actix-web/pull/1920


## 4.0.0-beta.7 - 2021-09-09
- Minimum supported Rust version (MSRV) is now 1.51.


## 4.0.0-beta.6 - 2021-06-26
- Update `actix` to `0.12`. [#2277]

[#2277]: https://github.com/actix/actix-web/pull/2277


## 4.0.0-beta.5 - 2021-06-17
- No notable changes.


## 4.0.0-beta.4 - 2021-04-02
- No notable changes.


## 4.0.0-beta.3 - 2021-03-09
- No notable changes.


## 4.0.0-beta.2 - 2021-02-10
- No notable changes.


## 4.0.0-beta.1 - 2021-01-07
- Update `pin-project` to `1.0`.
- Update `bytes` to `1.0`. [#1813]
- `WebsocketContext::text` now takes an `Into<bytestring::ByteString>`. [#1864]

[#1813]: https://github.com/actix/actix-web/pull/1813
[#1864]: https://github.com/actix/actix-web/pull/1864


## 3.0.0 - 2020-09-11
- No significant changes from `3.0.0-beta.2`.


## 3.0.0-beta.2 - 2020-09-10
- Update `actix-*` dependencies to latest versions.


## [3.0.0-beta.1] - 2020-xx-xx
- Update `actix-web` & `actix-http` dependencies to beta.1
- Bump minimum supported Rust version to 1.40


## [3.0.0-alpha.1] - 2020-05-08
- Update the actix-web dependency to 3.0.0-alpha.1
- Update the actix dependency to 0.10.0-alpha.2
- Update the actix-http dependency to 2.0.0-alpha.3

## [2.0.0] - 2019-12-20

- Release

## [2.0.0-alpha.1] - 2019-12-15

- Migrate to actix-web 2.0.0

## [1.0.4] - 2019-12-07

- Allow comma-separated websocket subprotocols without spaces (#1172)

## [1.0.3] - 2019-11-14

- Update actix-web and actix-http dependencies

## [1.0.2] - 2019-07-20

- Add `ws::start_with_addr()`, returning the address of the created actor, along
  with the `HttpResponse`.

- Add support for specifying protocols on websocket handshake #835

## [1.0.1] - 2019-06-28

- Allow to use custom ws codec with `WebsocketContext` #925

## [1.0.0] - 2019-05-29

- Update actix-http and actix-web

## [0.1.0-alpha.3] - 2019-04-02

- Update actix-http and actix-web

## [0.1.0-alpha.2] - 2019-03-29

- Update actix-http and actix-web

## [0.1.0-alpha.1] - 2019-03-28

- Initial impl
