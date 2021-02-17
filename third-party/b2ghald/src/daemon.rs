use b2ghald::backlight::Backlight;
use b2ghald::messages::*;
use bincode::Options;
use log::{error, info};
use nix::sys::reboot::{reboot, RebootMode};
use nix::sys::stat::{umask, Mode};
use std::io::{Error, ErrorKind};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

// Manages a session with a client.
fn handle_client(stream: UnixStream) -> Result<(), Error> {
    let config = bincode::DefaultOptions::new().with_native_endian();

    let backlight = Backlight::new();
    if let Ok(ref device) = backlight {
        info!("Backlight available at {:?}", device);
    }

    loop {
        match config.deserialize_from::<_, ToDaemon>(&stream) {
            Ok(message) => {
                let sock_copy = stream.try_clone().expect("Couldn't clone socket");

                macro_rules! send {
                    ($payload:expr) => {
                        let response = FromDaemon::new(message.id(), $payload);
                        config
                            .serialize_into(sock_copy, &response)
                            .map_err(|_| Error::new(ErrorKind::Other, "bincode error"))?
                    };
                }

                match message.request() {
                    Request::GetBrightness(screen_id) => {
                        info!("GetBrightness #{}", message.id());
                        let payload = match backlight {
                            Ok(ref device) => Response::GetBrightnessSuccess((
                                *screen_id,
                                device.get_brightness(*screen_id),
                            )),
                            Err(_) => Response::GetBrightnessError,
                        };
                        send!(payload);
                    }
                    Request::SetBrightness((screen_id, level)) => {
                        info!(
                            "SetBrightness {} on screen {} #{}",
                            level,
                            screen_id,
                            message.id()
                        );
                        let payload = match backlight {
                            Ok(ref device) => {
                                device.set_brightness(*screen_id, *level);
                                Response::SetBrightnessSuccess
                            }
                            Err(_) => Response::SetBrightnessError,
                        };
                        send!(payload);
                    }
                    Request::PowerOff => {
                        let _ = reboot(RebootMode::RB_POWER_OFF);
                        send!(Response::GenericSuccess);
                    }
                    Request::Reboot => {
                        let _ = reboot(RebootMode::RB_AUTOBOOT);
                        send!(Response::GenericSuccess);
                    }
                    Request::EnableScreen(value) => {
                        let payload = match backlight {
                            Ok(ref device) => {
                                device.enable_screen(*value);
                                Response::GenericSuccess
                            }
                            Err(_) => Response::GenericError,
                        };
                        send!(payload);
                    }
                    Request::DisableScreen(value) => {
                        let payload = match backlight {
                            Ok(ref device) => {
                                device.disable_screen(*value);
                                Response::GenericSuccess
                            }
                            Err(_) => Response::GenericError,
                        };
                        send!(payload);
                    }
                }
            }
            Err(err) => {
                match *err {
                    bincode::ErrorKind::Io(io_err) => {
                        info!("Client connection was closed: {}", io_err)
                    }
                    err => error!("Decoding error: {}", err),
                }
                break;
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    // Unix sockets inherits the permissions of the parent directory,
    // so we create it as drwxrwxrwx
    let mask = umask(Mode::empty());
    std::fs::DirBuilder::new()
        .recursive(true)
        .create("/tmp/b2g")?;

    let _ = std::fs::remove_file(b2ghald::SOCKET_PATH);
    let listener = UnixListener::bind(b2ghald::SOCKET_PATH)?;

    // Reset the umask for other operations in this process.
    umask(mask);

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                error!("Failed to get incoming client: {}", err);
                break;
            }
        }
    }
    Ok(())
}
