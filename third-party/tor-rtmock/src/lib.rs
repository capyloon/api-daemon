#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
//! Support for mocking with `tor-rtcompat` asynchronous runtimes.
//!
//! # Overview
//!
//! The `tor-rtcompat` crate defines a `Runtime` trait that represents
//! most of the common functionality of .  This crate provides mock
//! implementations that override a `Runtime`, in whole or in part,
//! for testing purposes.
//!
//! This crate is part of
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.
//! It is used to write tests for higher-level
//! crates in Arti that rely on asynchronous runtimes.
//!
//! This crate should only be used for writing tests.
//!
//! Currently, we support mocking the passage of time (via
//! [`MockSleepRuntime`]), and impersonating the internet (via
//! [`MockNetRuntime`]).
//!
//! # Examples
//!
//! Suppose you've written a function that relies on making a
//! connection to the network and possibly timing out:
//!
//! ```
//! use tor_rtcompat::{Runtime,SleepProviderExt};
//! use std::{net::SocketAddr, io::Result, time::Duration, io::Error};
//! use futures::io::AsyncWriteExt;
//!
//! async fn say_hi(runtime: impl Runtime, addr: &SocketAddr) -> Result<()> {
//!    let delay = Duration::new(5,0);
//!    runtime.timeout(delay, async {
//!       let mut conn = runtime.connect(addr).await?;
//!       conn.write_all(b"Hello world!\r\n").await?;
//!       conn.close().await?;
//!       Ok::<_,Error>(())
//!    }).await??;
//!    Ok(())
//! }
//! ```
//!
//! But how should you test this function?
//!
//! You might try connecting to a well-known website to test the
//! connection case, and to a well-known black hole to test the
//! timeout case... but that's a bit undesirable.  Your tests might be
//! running in a container with no internet access; and even if they
//! aren't, it isn't so great for your tests to rely on the actual
//! state of the internet.  Similarly, if you make your timeout too long,
//! your tests might block for a long time; but if your timeout is too short,
//! the tests might fail on a slow machine or on a slow network.
//!
//! Or, you could solve both of these problems by using `tor-rtmock`
//! to replace the internet _and_ the passage of time.  (Here we're only
//! replacing the internet.)
//!
//! ```
//! # use tor_rtcompat::{Runtime,SleepProviderExt};
//! # use std::{net::SocketAddr, io::Result, time::Duration, io::Error};
//! # use futures::io::AsyncWriteExt;
//! #
//! # async fn say_hi(runtime: impl Runtime, addr: &SocketAddr) -> Result<()> {
//! #   let delay = Duration::new(5,0);
//! #   runtime.timeout(delay, async {
//! #      let mut conn = runtime.connect(addr).await?;
//! #      conn.write_all(b"Hello world!\r\n").await?;
//! #      conn.close().await?;
//! #      dbg!("okay apparently");
//! #      Ok::<_,Error>(())
//! #   }).await??;
//! #   Ok(())
//! # }
//! use tor_rtmock::{MockSleepRuntime,MockNetRuntime,net::MockNetwork};
//! use tor_rtcompat::{TcpProvider,TcpListener};
//! use futures::io::AsyncReadExt;
//!
//! tor_rtcompat::test_with_all_runtimes!(|rt| async move {
//!
//!    let addr1 = "198.51.100.7".parse().unwrap();
//!    let addr2 = "198.51.100.99".parse().unwrap();
//!    let sockaddr = "198.51.100.99:101".parse().unwrap();
//!
//!    // Make a runtime that pretends that we are at the first address...
//!    let fake_internet = MockNetwork::new();
//!    let rt1 = fake_internet.builder().add_address(addr1).runtime(rt.clone());
//!    // ...and one that pretends we're listening at the second address.
//!    let rt2 = fake_internet.builder().add_address(addr2).runtime(rt);
//!    let listener = rt2.listen(&sockaddr).await.unwrap();
//!
//!    // Now we can test our function!
//!    let (result1,output) = futures::join!(
//!           say_hi(rt1, &sockaddr),
//!           async {
//!               let (mut conn,addr) = listener.accept().await.unwrap();
//!               assert_eq!(addr.ip(), addr1);
//!               let mut output = Vec::new();
//!               conn.read_to_end(&mut output).await.unwrap();
//!               output
//!           });
//!
//!    assert!(result1.is_ok());
//!    assert_eq!(&output[..], b"Hello world!\r\n");
//! });
//! ```
//!
//! (TODO: Add an example for the timeout case.)

// @@ begin lint list maintained by maint/add_warning @@
#![cfg_attr(not(ci_arti_stable), allow(renamed_and_removed_lints))]
#![cfg_attr(not(ci_arti_nightly), allow(unknown_lints))]
#![deny(missing_docs)]
#![warn(noop_method_call)]
#![deny(unreachable_pub)]
#![warn(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::checked_conversions)]
#![warn(clippy::cognitive_complexity)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::missing_panics_doc)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::let_unit_value)] // This can reasonably be done for explicitness
#![allow(clippy::significant_drop_in_scrutinee)] // arti/-/merge_requests/588/#note_2812945
//! <!-- @@ end lint list maintained by maint/add_warning @@ -->

extern crate core;

pub mod io;
pub mod net;
pub mod time;

mod net_runtime;
mod sleep_runtime;

pub use net_runtime::MockNetRuntime;
pub use sleep_runtime::MockSleepRuntime;
