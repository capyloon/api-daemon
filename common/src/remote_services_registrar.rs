/// Register remote services:
/// - list services.
/// - assign them a stable ID
/// - change ownership of their directory so they can read/write in it.

use log::error;
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Custom Error")]
    CustomError,
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("IO Error")]
    Io(#[from] io::Error),
}

type Result<T> = ::std::result::Result<T, Error>;

static APP_ID_BASE: u16 = 10000; // The first uid/gid used for services.

#[derive(Clone)]
pub struct RemoteServicesRegistrar {
    pub services: HashMap<String, u16>,
}

fn read_toml_file<P: AsRef<Path>>(config_path: P) -> Result<toml::value::Table> {
    let mut file = File::open(config_path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    toml::from_str(&source).map_err(|e| e.into())
}

fn write_as_toml_file<P: AsRef<Path>>(config_path: P, data: &HashMap<String, u16>) -> Result<()> {
    let mut file = File::create(config_path)?;
    for (name, id) in data {
        writeln!(&mut file, "{}={}", name, id)?;
    }
    Ok(())
}

impl RemoteServicesRegistrar {
    // Loads an existing service description file, and merge that
    // information with services discovered by listing a directory of
    // services.
    pub fn new<P: AsRef<Path>>(config_path: P, root_dir: P) -> Self {
        // Get the initial list from the configuration file.
        let mut max_id = 0;
        let config_services = if let Ok(toml) = read_toml_file(&config_path) {
            let mut result = HashMap::new();
            for (name, value) in toml {
                if let toml::Value::Integer(id) = value {
                    let id = id as u16;
                    if id > max_id {
                        max_id = id;
                    }
                    result.insert(name, id);
                }
            }
            result
        } else {
            HashMap::new()
        };

        // Iterate over the subdirectories to find services. If a given service
        // already exists, don't change its id. If it's a new, assign it a new
        // id which is the next largest id.
        let mut services = HashMap::new();
        if let Ok(dir) = read_dir(&root_dir) {
            for item in dir {
                if let Ok(entry) = item {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if !config_services.contains_key(&name) {
                                max_id += 1;
                                services.insert(name, max_id);
                            } else {
                                services.insert(name.clone(), *config_services.get(&name).unwrap());
                            }
                        }
                    }
                }
            }
        }

        // On device, chown and chmod the directory for each service.
        #[cfg(target_os = "android")]
        {
            use nix::unistd::{chown, Gid, Uid};
            use nix::sys::stat::{Mode, fchmodat, FchmodatFlags};
            for (service, id) in &services {
                let mut path = root_dir.as_ref().join(service);
                let id = (id + APP_ID_BASE).into();
                let uid = Uid::from_raw(id);
                let gid = Gid::from_raw(id);
                if let Err(err) = chown(&path, Some(uid), Some(gid)) {
                    error!("Failed to chown {:?} : {}", path, err);
                }
                let mode = Mode::from_bits(0o755).unwrap();
                if let Err(err) = fchmodat(None, &path, mode, FchmodatFlags::FollowSymlink) {
                    error!("Failed to fchmodat {:?} : {}", path, err);
                }
                // Also chown the daemon executable.
                path.push("daemon");
                if let Err(err) = chown(&path, Some(uid), Some(gid)) {
                    error!("Failed to chown {:?} : {}", path, err);
                }
            }
        }

        // Update the configuration file.
        #[cfg(not(test))]
        let write_path = config_path.as_ref();

        // When testing, don't overwrite the input file.
        #[cfg(test)]
        let s = format!("{}_test", config_path.as_ref().display());
        #[cfg(test)]
        let write_path = Path::new(&s);

        if let Err(err) = write_as_toml_file(&write_path, &services) {
            error!("Failed to write update service file at {} : {}", write_path.display(), err);
        }

        RemoteServicesRegistrar { services }
    }

    pub fn id_for(&self, name: &str) -> Option<u16> {
        self.services.get(name).map(|value| value + APP_ID_BASE)
    }
}

#[test]
fn init_remote_service_manager() {
    // Bad file config path.
    let manager = RemoteServicesRegistrar::new("./tests/no_such_file.toml", "");
    assert_eq!(manager.services.len(), 0);

    // Simple case with 2 services and matching directories.
    let manager = RemoteServicesRegistrar::new("./tests/services1.toml", "./tests/remote_services");
    assert_eq!(manager.services.len(), 2);

    // No matching directory for this config file.
    let manager = RemoteServicesRegistrar::new("./tests/services1.toml", "./tests/empty_remote_services");
    assert_eq!(manager.services.len(), 0);

    // New service available in the directory
    let manager = RemoteServicesRegistrar::new("./tests/services1.toml", "./tests/three_remote_services");
    assert_eq!(manager.services.len(), 3);
    assert_eq!(*manager.services.get("newservice").unwrap(), 3);

    // New service available in the directory, gap in the service id.
    let manager = RemoteServicesRegistrar::new("./tests/services2.toml", "./tests/three_remote_services");
    assert_eq!(manager.services.len(), 3);
    assert_eq!(*manager.services.get("newservice").unwrap(), 25);

    // Remove now useless files.
    use std::fs::remove_file;
    let _ = remove_file("./tests/no_such_file.toml_test");
    let _ = remove_file("./tests/services1.toml_test");
    let _ = remove_file("./tests/services2.toml_test");
}
