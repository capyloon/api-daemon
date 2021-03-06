#[cfg(target_os = "linux")]
mod linux;

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[allow(unused)]
use std::io;

#[allow(unused)]
#[track_caller]
fn expect_ok(result: io::Result<()>) {
    assert!(result.is_ok());
}

#[allow(unused)]
#[track_caller]
fn expect_err(result: io::Result<()>, kind: io::ErrorKind) {
    assert_eq!(result.unwrap_err().kind(), kind);
}

#[test]
fn tools_nofile() {
    let lim = rlimit::increase_nofile_limit(u64::MAX).unwrap();
    dbg!(lim);
}
