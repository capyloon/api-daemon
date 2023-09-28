# Migration guide

## [3.0.0]
- Default `impl-index-from` feature is now always enabled.
  Use wrappers for `TypedIndex` values
  if you use different `From/Into` `usize` and `TypedIndex::{from_usize, into_usize}` implementations.
- Trait `TypedIndex` was removed.
  Use `From<usize>` and `Into<usize>` instead.

## [2.0.0]
- Use `TiSlice::from_ref()`, `TiSlice::from_mut()`,
  `AsRef::as_ref()` and `AsMut::as_mut()` instead `Into::into()`
  for zero-cost conversions between `&slice` and `&TiSlice`, `&mut slice` and `&mut TiSlice`,
  `&std::Vec` and `&TiVec`, `&mut std::Vec` and `&TiVec`.

[3.0.0]: https://github.com/zheland/typed-index-collections/compare/v2.0.1...v3.0.0
[2.0.0]: https://github.com/zheland/typed-index-collections/compare/v1.1.0...v2.0.0
