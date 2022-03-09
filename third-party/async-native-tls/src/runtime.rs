#[cfg(feature = "runtime-async-std")]
pub(crate) use futures_util::io::{AsyncRead, AsyncWrite};

#[cfg(feature = "runtime-async-std")]
#[allow(unused_imports)]
pub(crate) use futures_util::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(feature = "runtime-tokio")]
pub(crate) use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
