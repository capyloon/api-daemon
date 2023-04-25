// #![feature(test)]
// extern crate test;

#[macro_use]
extern crate lazy_static;

pub mod array;
pub mod common;
pub mod config;
pub mod file_store;
pub mod fts;
pub mod http;
pub mod indexer;
pub mod manager;
pub mod scorer;
mod timer;
pub mod xor_store;
