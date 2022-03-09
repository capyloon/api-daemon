//! Detect a "ctrl-c" notification or other reason to exit.

use anyhow::Result;

/// Wait until a control-c notification is received, using an appropriate
/// runtime mechanism.
///
/// This function can have pretty kludgy side-effects: see
/// documentation for `tokio::signal::ctrl_c` and `async_ctrlc` for
/// caveats.  Notably, you can only call this once with async_std.
pub(crate) async fn wait_for_ctrl_c() -> Result<()> {
    #[cfg(feature = "tokio")]
    {
        tokio_crate::signal::ctrl_c().await?;
    }
    #[cfg(all(feature = "async-std", not(feature = "tokio")))]
    {
        async_ctrlc::CtrlC::new().unwrap().await;
    }
    Ok(())
}
