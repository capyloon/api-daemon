# Changelog

## 1.0.7

Fixes:

* Fixes broken 32bit builds by matching `timespec` defs to libc

## 1.0.6

Fixes:

* marks `Vnode` enum `non_exhaustive` to fix backwards compatibility in 1.x

## 1.0.5

Adds:

* docs.rs support
* added new enum variants specific to FreeBSD (broke backwards compatibility)

Fixes:

* Fixes broken 32bit builds

## 1.0.4

Fixes:

* Fixes broken NetBSD build

## 1.0.3

Adds:

* #6: Adds a new `Watcher.poll_forever()` method which blocks on new events. This works
  around buggy behavior in the original `Watcher.poll()` method.
* !3: Adds an implementation for `std::os::unix::io::AsRawFd` for `Watcher` for
  nested kqueues.

## 1.0.2

* Fixed #4: Fix bug where wrong data types were used on i386 FreeBSD

## 1.0.1

* Merged !1 as a fix for #3. We properly fill in the `ext` field for `kqueue`
  extensions on FreeBSD.

## 1.0.0

### Breaking changes

* Bumped `bitflags` in `rust-kqueue-sys`: Now all bitflag constants must be qualified:

`EV_DELETE` -> `EventFlag::EV_DELETE`
`NOTE_WRITE` > `FilterFlag::NOT_WRITE`

### Other changes

* 2018 edition and clippy changes
