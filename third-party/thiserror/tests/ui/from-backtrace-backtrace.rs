// https://github.com/dtolnay/thiserror/issues/163

#![feature(backtrace)]

use std::backtrace::Backtrace;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("...")]
pub struct Error(#[from] #[backtrace] std::io::Error, Backtrace);

fn main() {}
