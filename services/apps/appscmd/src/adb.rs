// ADB support

use super::CmdLineError;
use log::debug;
use mozdevice::{AndroidStorageInput, Device, DeviceError, Host};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::Path;

// Device only has tcp forwarding, so we use the lower lever api
// to forward to the unix socket.
fn forward_path_to_port(host: &Host, local: u16, remote: &str) -> Result<u16, AdbError> {
    let command = format!("forward:tcp:{};localfilesystem:{}", local, remote);
    let response = host.execute_host_command(&command, true, false)?;

    if local == 0 {
        Ok(response.parse::<u16>().unwrap_or(0))
    } else {
        Ok(local)
    }
}

#[derive(Debug)]
pub(crate) struct AdbError {
    inner: DeviceError,
}

impl From<DeviceError> for AdbError {
    fn from(inner: DeviceError) -> Self {
        Self { inner }
    }
}

impl fmt::Display for AdbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl Error for AdbError {}

pub(crate) struct AdbDevice {
    device: Device,
}

impl AdbDevice {
    pub(crate) fn new(
        port: u16,
        uds_path: &str,
        backup_path: Option<&str>,
    ) -> Result<Self, AdbError> {
        let host = Host {
            ..Default::default()
        };
        let device = host.device_or_default::<String>(None, AndroidStorageInput::Auto)?;
        debug!("Using device {}", device.serial);

        let uds_exist = device.path_exists(Path::new(uds_path), true)?;
        if uds_exist {
            forward_path_to_port(&device.host, port, uds_path)?;
        } else if let Some(path) = backup_path {
            forward_path_to_port(&device.host, port, path)?;
        }
        Ok(Self { device })
    }

    pub(crate) fn push(&self, source: &str, dest: &str) -> Result<(), CmdLineError> {
        let mut file = File::open(source)?;

        self.device
            .push(&mut file, Path::new(dest), 0o555)
            .map_err(AdbError::from)
            .map_err(|err| err.into())
    }
}
