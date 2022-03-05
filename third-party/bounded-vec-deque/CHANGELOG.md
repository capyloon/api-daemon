# Changelog
All notable changes to this project will be documented in this changeling.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html), with the additional guarantee
that patch versions will not introduce backward-incompatible changes even before v1.0.

## Unreleased

## 0.1.1 - 2019-07-08
### Added
+ This changelog.

### Fixed
+ A bug where a single item could be pushed or inserted into a `BoundedVecDeque` with a maximum
  length of zero. ([`45c3d7d4`](https://gitlab.com/Moongoodboy/bounded-vec-deque/commit/45c3d7d4))

## 0.1.0 - 2019-04-07
### Added
+ The `BoundedVecDeque` type, this crate's raison d'être.
+ Five supporting iterator types (`Iter`, `IterMut`, `Drain`, `Append`, `IntoIter`), returned by
  various `BoundedVecDeque` methods.
