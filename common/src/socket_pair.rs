//! A simple abstraction over UnixDatagram pairs
//! providing Read and Write implementations.

use std::io::{self, Error as IoError, Result as IoResult};
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixDatagram;

#[derive(Clone, Debug)]
pub struct SocketPair(pub PairedStream, pub PairedStream);

#[derive(Debug)]
pub struct PairedStream {
    fd: UnixDatagram,
}

impl Read for PairedStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.fd.recv(buf)
    }
}

impl Write for PairedStream {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.fd.send(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl Clone for PairedStream {
    fn clone(&self) -> Self {
        match self.fd.try_clone() {
            Ok(fd) => PairedStream { fd },
            Err(err) => {
                log::error!(
                    "PairedStream cloning fd {} failed: {}",
                    self.fd.as_raw_fd(),
                    err
                );
                panic!(
                    "PairedStream cloning fd {} failed: {}",
                    self.fd.as_raw_fd(),
                    err
                );
            }
        }
    }
}

impl FromRawFd for PairedStream {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        PairedStream {
            fd: UnixDatagram::from_raw_fd(fd),
        }
    }
}

impl PairedStream {
    fn new(fd: UnixDatagram) -> Self {
        PairedStream { fd }
    }

    pub fn raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl SocketPair {
    pub fn new() -> Result<Self, IoError> {
        match UnixDatagram::pair() {
            Ok((child_fd, parent_fd)) => Ok(SocketPair(
                PairedStream::new(child_fd),
                PairedStream::new(parent_fd),
            )),
            Err(err) => Err(IoError::new(io::ErrorKind::Other, format!("{}", err))),
        }
    }
}

#[test]
fn socket_pair_echo() {
    let pair = SocketPair::new().unwrap();
    let mut parent = pair.0;
    let mut child = pair.1;

    let buf = [9, 8, 7, 6, 5, 4, 3, 2, 1];

    parent.write(&buf).unwrap();

    let mut response = [0; 9];
    child.read(&mut response).unwrap();

    assert_eq!(buf, response);

    child.write(b"Hello World").unwrap();

    let mut response = [0; 11];
    parent.read(&mut response).unwrap();

    let s = String::from_utf8_lossy(&response);
    assert_eq!(s, "Hello World".to_owned());
}

// We use fork() to test multiple processes since that's easier to setup
// in tests that using several test executables.
#[test]
fn socket_pair_fork() {
    use nix::unistd::{fork, ForkResult};

    let pair = SocketPair::new().unwrap();
    let mut parent = pair.0;
    let mut child = pair.1;

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { .. }) => {
                parent.write(b"From Parent").unwrap();

                let mut response = [0; 10];
                parent.read(&mut response).unwrap();
                let s = String::from_utf8_lossy(&response);
                assert_eq!(s, "From Child".to_owned());
            }
            Ok(ForkResult::Child) => {
                let mut response = [0; 11];
                child.read(&mut response).unwrap();
                let s = String::from_utf8_lossy(&response);
                assert_eq!(s, "From Parent".to_owned());

                child.write(b"From Child").unwrap();
            }
            Err(_) => panic!("Fork failed"),
        }
    }
}

#[test]
fn socket_pair_bincode() {
    // Tests sending and receiving bincode encoded structs.

    use nix::unistd::{fork, ForkResult};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestMessage {
        msg: String,
        id: u32,
        error: bool,
    }

    let pair = SocketPair::new().unwrap();
    let parent = pair.0;
    let child = pair.1;

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { .. }) => {
                let msg1 = TestMessage {
                    msg: "From Parent".into(),
                    id: 42,
                    error: false,
                };
                let _ = bincode::serialize_into(parent.clone(), &msg1);

                let response: TestMessage = bincode::deserialize_from(parent).unwrap();

                assert_eq!(response.msg, "From Child".to_owned());
                assert_eq!(response.id, 21);
                assert_eq!(response.error, true);
            }
            Ok(ForkResult::Child) => {
                {
                    let response: TestMessage = bincode::deserialize_from(child.clone()).unwrap();
                    assert_eq!(response.msg, "From Parent".to_owned());
                    assert_eq!(response.id, 42);
                    assert_eq!(response.error, false);
                }

                let msg2 = TestMessage {
                    msg: "From Child".into(),
                    id: 21,
                    error: true,
                };
                let _ = bincode::serialize_into(child, &msg2);
            }
            Err(_) => panic!("Fork failed"),
        }
    }
}
