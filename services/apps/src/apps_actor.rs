use crate::apps_item::AppsItem;
use crate::apps_registry::AppsRegistry;
use crate::apps_storage::{validate_package, PackageError};
use crate::generated::common::*;
use crate::manifest::{Manifest, ManifestError};
use crate::shared_state::AppsSharedData;
use common::traits::Shared;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppsActorError {
    #[error("AppNotFound")]
    AppNotFound,
    #[error("AppsServiceError {:?}", 0)]
    ServiceError(AppsServiceError),
    #[error("Package missing")]
    PackageMissing,
    #[error("Installation directory not found")]
    InstallationDirNotFound,
    #[error("Empty url")]
    EmptyUrl,
    #[error("Invalid app name")]
    InvalidAppName,
    #[error("File copy failed")]
    FileCopyError,
    #[error("Directory creation failed")]
    DirCreationFail,
    #[error("Package corrupted, `{0}`")]
    WrongPackage(PackageError),
    #[error("Installation RegistrationError")]
    WrongRegistration,
    #[error("Installation Manifest Error, `{0}`")]
    WrongManifest(ManifestError),
    #[error("Io error, {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum HandleClientError {
    #[error("Failed to read from socket")]
    ReadSocketError,
    #[error("Command format error")]
    WrongCMD,
    #[error("Working directory not found")]
    WrongENV,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    cmd: String,
    param: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Response {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct FileRemover {
    pub path: String,
}

impl Drop for FileRemover {
    fn drop(&mut self) {
        let _ = fs::remove_file(PathBuf::from(&self.path).as_path());
    }
}

pub fn start_webapp_actor(shared_data: Shared<AppsSharedData>) {
    debug!("Starting apps actor");
    let path = {
        let shared = shared_data.lock();
        shared.config.uds_path.clone()
    };

    if path.is_empty() {
        error!("apps service unix socket path is not configured");
        return;
    }

    // Make sure the listener doesn't already exist.
    let _ = ::std::fs::remove_file(&path);

    match UnixListener::bind(&path) {
        Ok(listener) => {
            debug!("Starting thread of socket server in apps service");
            for stream in listener.incoming() {
                let shared_state = shared_data.clone();
                match stream {
                    Ok(stream) => {
                        debug!("Starting : thread incoming OK");
                        thread::spawn(move || handle_client(shared_state, stream));
                    }
                    Err(err) => {
                        error!("Error: {}", err);
                        break;
                    }
                }
            }
        }
        Err(err) => error!("Failed to bind apps actor socket at {} : {}", path, err),
    }
}

fn validate_request(request: &Request) -> bool {
    if (request.cmd == "install" || request.cmd == "install-pwa" || request.cmd == "uninstall")
        && request.param.is_some()
    {
        true
    } else {
        request.cmd == "list"
    }
}

fn handle_client(shared_data: Shared<AppsSharedData>, stream: UnixStream) {
    let mut stream_write = match stream.try_clone() {
        Ok(stream) => stream,
        Err(err) => {
            error!("Failed to clone uds stream: {}", err);
            return;
        }
    };

    let stream_read = BufReader::new(stream);
    for line in stream_read.lines() {
        if line.is_err() {
            error!("Failed to read lines");
            write_response(
                &mut stream_write,
                "unknown",
                false,
                &format!("{}", HandleClientError::ReadSocketError),
            );
            continue;
        }
        let line_w = line.as_ref().unwrap();
        let request = match serde_json::from_str::<Request>(&line_w) {
            Ok(request) => {
                debug!("{:?}", request.clone());
                if !validate_request(&request) {
                    error!("Invalid parameter");
                    write_response(
                        &mut stream_write,
                        "command",
                        false,
                        &format!("{}", HandleClientError::WrongCMD),
                    );
                    continue;
                }
                request
            }
            Err(err) => {
                error!("Failed to parse command {}", err);
                write_response(
                    &mut stream_write,
                    "command",
                    false,
                    &format!("{}", HandleClientError::WrongCMD),
                );
                continue;
            }
        };

        debug!("cmd {}, param path {:?}", request.cmd, request.param);
        if &request.cmd == "install" {
            let file = FileRemover {
                path: request.param.clone().unwrap(),
            };
            let manifest = match validate_package(&file.path) {
                Ok(m) => m,
                Err(err) => {
                    write_response(&mut stream_write, &request.cmd, false, &format!("{}", err));
                    continue;
                }
            };

            // Delete it at drop
            let from_file = Path::new(&file.path);
            if let Err(err) = install_package(&shared_data, &from_file, &manifest) {
                error!("Installation fails, {}", err);
                write_response(&mut stream_write, &request.cmd, false, &format!("{}", err));
                continue;
            }

            info!("Installation success");
            debug!("{:?} is valid", &request.param);
            write_response(&mut stream_write, &request.cmd, true, "success");
        }

        if &request.cmd == "install-pwa" {
            // request.param is assured to be Some.
            let url = request.param.clone().unwrap();
            if let Err(err) = install_pwa(&shared_data, &url) {
                error!("Installation fails, {}", err);
                write_response(
                    &mut stream_write,
                    &request.cmd,
                    false,
                    &format!("{:?}", err),
                );
                continue;
            }

            info!("Installation {:?} success", &request.param);
            write_response(&mut stream_write, &request.cmd, true, "success");
        }

        if &request.cmd == "uninstall" {
            if let Err(err) = uninstall(&shared_data, &request.param.unwrap()) {
                error!("Uninstallation fails, {}", err);
                write_response(&mut stream_write, &request.cmd, false, &format!("{}", err));

                continue;
            }

            info!("Uninstallation success");
            write_response(&mut stream_write, &request.cmd, true, "success");
        }

        if &request.cmd == "list" {
            match get_all(&shared_data) {
                Err(_) => {
                    error!("List application failed");
                    write_response(&mut stream_write, &request.cmd, false, "");

                    continue;
                }
                Ok(app_list) => {
                    debug!("List application success");
                    write_response(&mut stream_write, &request.cmd, true, &app_list);
                }
            }
        }
    }
}

pub fn install_pwa(shared_data: &Shared<AppsSharedData>, url: &str) -> Result<(), AppsActorError> {
    let mut shared = shared_data.lock();
    let data_path = shared.config.data_path.clone();
    let app = shared
        .registry
        .download_and_apply_pwa(&data_path, url)
        .map_err(AppsActorError::ServiceError)?;
    shared
        .registry
        .event_broadcaster
        .broadcast_app_installed(app);

    Ok(())
}

// Read valid application from from_path.
// Install it to data_dir.
// Update shared_data with manifest
pub fn install_package(
    shared_data: &Shared<AppsSharedData>,
    from_path: &Path,
    manifest: &Manifest,
) -> Result<(), AppsActorError> {
    let data_path = {
        let shared = shared_data.lock();
        shared.config.data_path.clone()
    };

    let path = Path::new(&data_path);

    if !from_path.is_file() {
        return Err(AppsActorError::PackageMissing);
    }
    if !path.is_dir() {
        return Err(AppsActorError::InstallationDirNotFound);
    }
    let mut shared = shared_data.lock();
    let app_name = AppsRegistry::sanitize_name(&manifest.get_name());
    if app_name.is_empty() {
        return Err(AppsActorError::InvalidAppName);
    }
    let update_url = format!("http://{}.localhost/manifest.webapp", &app_name);
    let app_name = shared
        .registry
        .get_unique_name(&manifest.get_name(), &update_url)
        .map_err(|_| AppsActorError::InvalidAppName)?;

    let download_dir = path.join(&format!("downloading/{}", &app_name));
    let download_app = download_dir.join("application.zip");

    fs::create_dir_all(download_dir.as_path())?;
    fs::copy(from_path, download_app.as_path())?;

    let is_update = shared.registry.get_by_update_url(&update_url).is_some();
    let vhost_port = shared.registry.get_vhost_port();
    // Need create appsItem object and add to db to reflect status
    let mut apps_item = AppsItem::default(&app_name, vhost_port);
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    }
    apps_item.set_update_url(&update_url);
    apps_item.set_install_state(AppsInstallState::Installing);
    let _ = shared
        .registry
        .apply_download(&mut apps_item, &download_dir, &manifest, &path, is_update)
        .map_err(|_| AppsActorError::WrongRegistration)?;

    if is_update {
        shared
            .registry
            .event_broadcaster
            .broadcast_app_updated(AppsObject::from(&apps_item));

        shared.vhost_api.app_updated(&app_name);
    } else {
        shared
            .registry
            .event_broadcaster
            .broadcast_app_installed(AppsObject::from(&apps_item));

        shared.vhost_api.app_installed(&app_name);
    }

    Ok(())
}

pub fn uninstall(
    shared_data: &Shared<AppsSharedData>,
    manifest_url: &str,
) -> Result<(), AppsActorError> {
    if manifest_url.is_empty() {
        return Err(AppsActorError::EmptyUrl);
    }

    let mut shared = shared_data.lock();
    let app = match shared.get_by_manifest_url(manifest_url) {
        Ok(app) => app,
        Err(err) => {
            error!("Do not find uninstall app: {:?}", err);
            return Err(AppsActorError::AppNotFound);
        }
    };
    let data_path = shared.config.data_path.clone();

    let _ = shared
        .registry
        .uninstall_app(&app.name, &app.update_url, &data_path)
        .map_err(|_| AppsActorError::WrongRegistration)?;

    shared
        .registry
        .event_broadcaster
        .broadcast_app_uninstalled(manifest_url.into());

    shared.vhost_api.app_uninstalled(&app.name);

    Ok(())
}

pub fn get_all(shared_data: &Shared<AppsSharedData>) -> Result<String, ()> {
    let shared = shared_data.lock();
    match shared.get_all_apps() {
        Ok(apps) => {
            if apps.is_empty() {
                info!("Empty application list");
                return Ok("".to_string());
            }
            let apps_str = serde_json::to_string(&apps).map_err(|_| ())?;
            debug!("serialized apps is {}", apps_str);

            Ok(apps_str)
        }
        Err(err) => {
            error!("{:?}", err);
            Err(())
        }
    }
}

fn write_response(s: &mut UnixStream, name: &str, success: bool, result: &str) {
    let resp = Response {
        name: name.into(),
        success: if success { Some(result.into()) } else { None },
        error: if !success { Some(result.into()) } else { None },
    };
    let mut resp_string = serde_json::to_string(&resp).unwrap();
    resp_string.push_str("\r\n");
    debug!("serialized response {}", resp_string);
    let _ = s.write_all(resp_string.as_bytes());
}

#[test]
fn test_manifest() {
    use std::env;
    use zip::result::ZipError;

    let _ = env_logger::try_init();
    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();

    // Normal case
    {
        let app_zip = format!(
            "{}/test-fixtures/apps-from/success/application.zip",
            current.display()
        );
        match validate_package(&app_zip) {
            Ok(_) => {
                assert!(true);
            }
            Err(err) => {
                assert!(false);
                println!("{}2", err);
            }
        }
    }
    // ZipPackageNotFound
    {
        let app_zip = format!(
            "{}/test-fixtures/apps-from/missing/application.zip",
            current.display()
        );
        match validate_package(&app_zip) {
            Ok(_) => {
                assert!(false);
            }
            Err(err) => assert_eq!(
                format!("{}", err),
                format!("{}", "Io error, No such file or directory (os error 2)")
            ),
        }
    }

    // ManifestMissing
    {
        let app_zip = format!(
            "{}/test-fixtures/apps-from/wrong-manifest/application.zip",
            current.display()
        );
        match validate_package(&app_zip) {
            Ok(_) => {
                assert!(false);
            }
            Err(err) => {
                assert_eq!(
                    format!("{}", err),
                    format!("{}", PackageError::FromZipError(ZipError::FileNotFound))
                );
            }
        }
    }

    // NameMissing
    {
        let app_zip = format!(
            "{}/test-fixtures/apps-from/wrong-manifest/application1.zip",
            current.display()
        );
        match validate_package(&app_zip) {
            Ok(_) => {
                assert!(false);
            }
            Err(err) => {
                assert_eq!(
                    format!("{}", err),
                    format!(
                        "{}",
                        PackageError::WrongManifest(ManifestError::NameMissing)
                    )
                );
            }
        }

        {
            let app_zip = format!(
                "{}/test-fixtures/apps-from/wrong-manifest/application1_1.zip",
                current.display()
            );
            match validate_package(&app_zip) {
                Ok(_) => {
                    assert!(false);
                }
                Err(err) => {
                    assert_eq!(format!("{}", err),
                        String::from("Package Manifest Error, Json Error missing field `name` at line 26 column 1"));
                }
            }
        }
    }

    // FromZipError
    {
        let app_zip = format!(
            "{}/test-fixtures/apps-from/wrong-zip-format/application.zip",
            current.display()
        );
        match validate_package(&app_zip) {
            Ok(_) => {
                assert!(false);
            }
            Err(err) => {
                assert_eq!(
                    format!("{}", err),
                    format!(
                        "{}",
                        PackageError::FromZipError(ZipError::InvalidArchive("Invalid zip header"))
                    )
                );
            }
        }
    }
}

#[cfg(test)]
fn test_install_app() {
    use crate::config;
    use crate::service::AppsService;
    use common::traits::Service;
    use config::Config;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!("{}/test-fixtures/test-apps-dir-install", current.display());
    let _test_path = Path::new(&_test_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    if let Err(err) = fs::remove_dir_all(&_test_path) {
        println!("test_install_app error: {:?}", err);
        assert!(true);
    }
    if let Err(err) = fs::create_dir_all(&_test_path) {
        println!("test_install_app error: {:?}", err);
        assert!(true);
    }

    let src_app = current.join("test-fixtures/apps-from/helloworld/application.zip");
    println!("src_app: {}", &src_app.display());

    // Test from shared object
    {
        let shared_data = AppsService::shared_state();
        let config = Config {
            root_path: _root_dir.clone(),
            data_path: _test_dir.clone(),
            uds_path: String::from("uds_path"),
            cert_type: String::from("test"),
        };
        {
            let mut shared = shared_data.lock();
            shared.config = config.clone();

            let registry = AppsRegistry::initialize(&config, 4443).unwrap();
            shared.registry = registry;
            println!("shared.apps_objects.len: {}", shared.registry.count());
            assert_eq!(4, shared.registry.count());
        }

        let app_name: String = "helloworld".into();
        let manifest = validate_package(&src_app.as_path()).unwrap();
        let milisec_before_installing = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Ok(_) = install_package(&shared_data, &src_app.as_path(), &manifest) {
            println!("App installed");
            assert!(true);
        } else {
            println!("App installed failed");
            assert!(false);
        }
        {
            let shared = shared_data.lock();
            println!(
                "After installed, shared.apps_objects.len: {}",
                shared.registry.count()
            );
            assert_eq!(5, shared.registry.count());

            match shared.registry.get_first_by_name(&app_name) {
                Some(app) => {
                    println!("Installation, success");
                    println!("app.get_install_time() {:?}", app.get_install_time());
                    println!("milisec_before_installing {:?}", milisec_before_installing);
                    assert_eq!(true, app.get_install_time() >= milisec_before_installing);
                }
                None => {
                    println!("Installation, failed");
                    assert!(false);
                }
            }

            println!("shared.apps_objects.len: {}", shared.registry.count());
            assert_eq!(5, shared.registry.count());
        }

        // Re-install
        let manifest = validate_package(&src_app.as_path()).unwrap();
        let milisec_before_installing1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if let Ok(_) = install_package(&shared_data, &src_app.as_path(), &manifest) {
            println!("App re-installed");
            assert!(true);
        } else {
            println!("App re-installed failed");
            assert!(false);
        }
        {
            let shared = shared_data.lock();
            match shared.registry.get_first_by_name(&app_name) {
                Some(app) => {
                    println!("Re-Installation, success");
                    assert_eq!(true, app.get_install_time() >= milisec_before_installing1);
                }
                None => {
                    println!("Installation, failed");
                    assert!(false);
                }
            }

            println!(
                "After re-installed, shared.apps_objects.len: {}",
                shared.registry.count()
            );
            assert_eq!(5, shared.registry.count());
        }
    }

    // Test by reloading data from persisted storage.
    {
        if !_test_path.is_dir() {
            println!("Webapp dir does not exist.");
            assert!(false);
        }

        let shared_data = AppsService::shared_state();
        let config = Config {
            root_path: _root_dir.clone(),
            data_path: _test_dir.clone(),
            uds_path: String::from("uds_path"),
            cert_type: String::from("test"),
        };
        {
            let mut shared = shared_data.lock();
            shared.config = config.clone();

            let registry = AppsRegistry::initialize(&config, 4443).unwrap();
            shared.registry = registry;
            println!(
                "Test from persisted storage, len: {}",
                shared.registry.count()
            );
            assert_eq!(5, shared.registry.count());
        }

        let mut manifest_url = String::new();
        {
            let shared = shared_data.lock();
            let app_name: String = "helloworld".into();
            let vhost_port = shared.registry.get_vhost_port();
            let apps_item = AppsItem::default(&app_name, vhost_port);
            manifest_url = apps_item.get_manifest_url();
            if let Ok(app) = shared.get_by_manifest_url(&manifest_url) {
                assert_eq!(app_name, app.name);
            } else {
                println!("get_by_manifest_url failed.");
                assert!(false);
            }
        }

        // Uninstall
        if let Ok(_) = uninstall(&shared_data, &manifest_url) {
            let shared = shared_data.lock();
            println!(
                "After uninstall, shared.apps_objects.len: {}",
                shared.registry.count()
            );
            assert_eq!(4, shared.registry.count());
        } else {
            println!("uninstall failed");
            assert!(false);
        }
    }
}

#[cfg(test)]
fn test_get_all() {
    use crate::config;
    use crate::service::AppsService;
    use common::traits::Service;
    use config::Config;
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let _root_dir = format!("{}/test-fixtures/webapps", current.display());
    let _test_dir = format!("{}/test-fixtures/test-apps-dir-get-all", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    if let Err(err) = fs::remove_dir_all(Path::new(&_test_dir)) {
        println!("test_get_all error: {:?}", err);
        assert!(true);
    }

    if let Err(err) = fs::create_dir_all(PathBuf::from(_test_dir.clone())) {
        println!("test_get_all error: {:?}", err);
        assert!(true);
    }

    println!("Register from: {}", &_root_dir);

    println!("test_get_all dir: {}", &_test_dir);
    let shared_data = AppsService::shared_state();
    let config = Config {
        root_path: _root_dir.clone(),
        data_path: _test_dir.clone(),
        uds_path: String::from("uds_path"),
        cert_type: String::from("test"),
    };
    {
        let mut shared = shared_data.lock();
        shared.config = config.clone();

        let registry = match AppsRegistry::initialize(&config, 8443) {
            Ok(registry) => registry,
            Err(err) => {
                println!("AppsRegistry::initialize error: {:?}", err);
                return assert!(true);
            }
        };
        shared.registry = registry;
        shared.state = AppsServiceState::Running;
    }

    let app_list = get_all(&shared_data).unwrap();
    let expected = "[{\"name\":\"calculator\",\"install_state\":\"Installed\",\"manifest_url\":\"http://calculator.localhost:8443/manifest.webapp\",\"status\":\"Enabled\",\"update_state\":\"Idle\",\"update_url\":\"https://store.server/calculator/manifest.webapp\",\"allowed_auto_download\":false},{\"name\":\"system\",\"install_state\":\"Installed\",\"manifest_url\":\"http://system.localhost:8443/manifest.webapp\",\"status\":\"Enabled\",\"update_state\":\"Idle\",\"update_url\":\"https://store.server/system/manifest.webapp\",\"allowed_auto_download\":false},{\"name\":\"gallery\",\"install_state\":\"Installed\",\"manifest_url\":\"http://gallery.localhost:8443/manifest.webapp\",\"status\":\"Enabled\",\"update_state\":\"Idle\",\"update_url\":\"https://store.server/gallery/manifest.webapp\",\"allowed_auto_download\":false},{\"name\":\"launcher\",\"install_state\":\"Installed\",\"manifest_url\":\"http://launcher.localhost:8443/manifest.webapp\",\"status\":\"Enabled\",\"update_state\":\"Idle\",\"update_url\":\"\",\"allowed_auto_download\":false}]";

    assert_eq!(app_list, expected);
}

#[test]
fn test_shared_state() {
    // Both test_get_all and test_install_app use shared_state.
    // To avoid the inconsistent result, run them in sequence.
    test_get_all();
    test_install_app();
}

#[test]
fn test_validate_request() {
    let req = Request {
        cmd: "install".into(),
        param: Some("path/to/app".into()),
    };
    assert!(validate_request(&req));

    let req = Request {
        cmd: "install".into(),
        param: Some("".into()),
    };
    assert!(validate_request(&req));

    let req = Request {
        cmd: "install".into(),
        param: None,
    };
    assert!(!validate_request(&req));

    let req = Request {
        cmd: "uninstall".into(),
        param: Some("app".into()),
    };
    assert!(validate_request(&req));

    let req = Request {
        cmd: "uninstall".into(),
        param: Some("".into()),
    };
    assert!(validate_request(&req));

    let req = Request {
        cmd: "uninstall".into(),
        param: None,
    };
    assert!(!validate_request(&req));

    let req = Request {
        cmd: "list".into(),
        param: None,
    };
    assert!(validate_request(&req));

    let req = Request {
        cmd: "list".into(),
        param: Some("none".into()),
    };
    assert!(validate_request(&req));
}
