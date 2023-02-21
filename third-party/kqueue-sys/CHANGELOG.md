# Changelog

## 1.0.3

* #1: fixes compiler error on NetBSD

## 1.0.2

* !2: fixes compiler error on darwin

## 1.0.1

* !1: fixes bug where watching multiple files on FreeBSD would fail

## 1.0.0

### Breaking

* `kevent.data` changed from `int64_t` -> `i64`
* Bumped `bitflags`: Now all bitflag constants must be qualified:

`EV_DELETE` -> `EventFlag::EV_DELETE`
`NOTE_WRITE` > `FilterFlag::NOT_WRITE`

### Others

* Updated to rust edition 2018
* Various clippy warnings
