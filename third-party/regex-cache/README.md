regex-cache [![Crates.io](https://img.shields.io/crates/v/regex-cache.svg)](https://crates.io/crates/regex-cache) [![regex-cache](https://docs.rs/regex-cache/badge.svg)](https://docs.rs/regex-cache) ![MIT](http://img.shields.io/badge/license-MIT-blue.svg) [![Build Status](https://travis-ci.org/1aim/rust-regex-cache.svg?branch=master)](https://travis-ci.org/1aim/rust-regex-cache)
===========
This crate provides a library for caching or lazily creating regular
expressions.

Lazy regular expressions are backed by a [OnceMutex](https://github.com/reem/rust-once-mutex),
while the regular expression cache is backed by a Least Recently Used cache.

Why not `lazy_static!`?
-----------------------
`lazy_static!` is great but it's only usable when you know what your regular
expression is.

In some cases you're loading an insane number of regular expressions (looking
at you libphonenumber) that you might eventually use but don't actually want to
store as `String` since that makes you lose type information and ease of use.

When to use `LazyRegex`
-----------------------
When you have many regular expressions you don't want to use instantly and
don't care about delaying the regular expression compilation.

When to use `RegexCache`
------------------------
When you want to limit the number of active regular expressions, the
`RegexCache` only keeps around a specified number of actively used regular
expressions.

Since it's an LRU cache having a small limit and using different regular
expressions every time ends up wasting memory and time for nothing.

When to use `CachedRegex`
-------------------------
When you want to transparently use a `RegexCache` and still have an easy to use
and typed regular expression instead of storing a `String` to then compile with
the `RegexCache`.

Keep in mind what's stored inside a `CachedRegex` is an
`Arc<Mutex<RegexCache>>`, which means there is locking involved when calling
any methods on `CachedRegex`.

For technical reasons there is no way to get a `&Regex` out of a `CachedRegex`
and some methods from `Regex` aren't usable on a `CachedRegex`, it should still
be fine for most uses.
