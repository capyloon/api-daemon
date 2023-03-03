use crate::apps_item::AppsItem;
use crate::apps_request::AppsRequest;
use crate::apps_storage::{validate_package, AppsStorage, PackageError};
use crate::downloader::{Downloader, DownloaderInfo};
use crate::generated::common::*;
use crate::manifest::{Manifest, ManifestError};
use crate::shared_state::AppsSharedData;
use common::traits::{Shared, SharedServiceState};
use geckobridge::service::GeckoBridgeService;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use url::Url;

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
    #[error("Internal Error")]
    Internal,
    #[error("Dependencies Error")]
    DependenciesError,
    #[error("DeepLink Error")]
    DeepLinkError,
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
    path: PathBuf,
}

impl FileRemover {
    fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }
}

impl Drop for FileRemover {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct RestartChecker {
    need_restart: bool,
    shared_state: Shared<AppsSharedData>,
}

impl Drop for RestartChecker {
    fn drop(&mut self) {
        self.shared_state
            .lock()
            .registry
            .check_need_restart(self.need_restart);
    }
}

pub fn start_webapp_actor(shared_data: Shared<AppsSharedData>) {
    use std::fs::{remove_file, set_permissions, Permissions};
    use std::os::unix::fs::PermissionsExt;

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
    let _ = remove_file(&path);

    match UnixListener::bind(&path) {
        Ok(listener) => {
            debug!("Starting thread of socket server in apps service");
            // Make the socket path rw for all to allow non-root adbd to connect.
            if let Err(err) = set_permissions(&path, Permissions::from_mode(0o666)) {
                error!("Failed to set 0o666 permission on {} : {}", path, err);
            }

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        debug!("Starting : thread incoming OK");
                        let shared_client = shared_data.clone();
                        thread::spawn(move || handle_client(shared_client, stream));
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
    if (request.cmd == "install"
        || request.cmd == "install-pwa"
        || request.cmd == "uninstall"
        || request.cmd == "launch")
        && request.param.is_some()
    {
        true
    } else {
        request.cmd == "list" || request.cmd == "ready" || request.cmd == "show-warnings"
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
        let request = match serde_json::from_str::<Request>(line_w) {
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
        match request.cmd.as_str() {
            "install" => {
                let file = FileRemover::new(&request.param.unwrap());
                let mut checker = RestartChecker {
                    need_restart: false,
                    shared_state: shared_data.clone(),
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
                match install_package(&shared_data, from_file, &manifest) {
                    Ok((_, need_restart)) => checker.need_restart = need_restart,
                    Err(err) => {
                        error!("Installation fails, {}", err);
                        write_response(&mut stream_write, &request.cmd, false, &format!("{}", err));
                        continue;
                    }
                }

                info!("Installation success");
                write_response(&mut stream_write, &request.cmd, true, "success");
            }
            "install-pwa" => {
                // request.param is assured to be Some.
                let url = match Url::parse(&request.param.unwrap()) {
                    Ok(url) => url,
                    Err(err) => {
                        error!("Invalid URL, error: {:?}", err);
                        write_response(
                            &mut stream_write,
                            &request.cmd,
                            false,
                            &format!("{:?}", err),
                        );
                        continue;
                    }
                };

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

                info!("Installation success");
                write_response(&mut stream_write, &request.cmd, true, "success");
            }
            "uninstall" => {
                let url = match Url::parse(&request.param.unwrap()) {
                    Ok(url) => url,
                    Err(err) => {
                        error!("Invalid URL,  error: {:?}", err);
                        write_response(
                            &mut stream_write,
                            &request.cmd,
                            false,
                            &format!("{:?}", err),
                        );
                        continue;
                    }
                };
                if let Err(err) = uninstall(&shared_data, &url) {
                    error!("Uninstallation fails, {}", err);
                    write_response(&mut stream_write, &request.cmd, false, &format!("{}", err));

                    continue;
                }

                info!("Uninstallation success");
                write_response(&mut stream_write, &request.cmd, true, "success");
            }
            "launch" => {
                let url = match Url::parse(&request.param.unwrap()) {
                    Ok(url) => url,
                    Err(err) => {
                        error!("Invalid URL,  error: {:?}", err);
                        write_response(
                            &mut stream_write,
                            &request.cmd,
                            false,
                            &format!("{:?}", err),
                        );
                        continue;
                    }
                };
                if let Err(err) = launch(&shared_data, &url) {
                    error!("launch failed, {}", err);
                    write_response(&mut stream_write, &request.cmd, false, &format!("{}", err));

                    continue;
                }

                info!("Launch success");
                write_response(&mut stream_write, &request.cmd, true, "success");
            }
            "list" => match get_all(&shared_data) {
                Err(_) => {
                    error!("List application failed");
                    write_response(&mut stream_write, &request.cmd, false, "");

                    continue;
                }
                Ok(app_list) => {
                    debug!("List application success");
                    write_response(&mut stream_write, &request.cmd, true, &app_list);
                }
            },
            "show-warnings" => {
                let warnings = AppsStorage::read_warnings();
                write_response(&mut stream_write, &request.cmd, true, &warnings);
            }
            "ready" => {
                let success = shared_data.lock().state == AppsServiceState::Running;
                write_response(&mut stream_write, &request.cmd, success, "");
            }
            _ => {}
        }
    }
}

pub fn install_pwa(shared_data: &Shared<AppsSharedData>, url: &Url) -> Result<(), AppsActorError> {
    let mut request =
        AppsRequest::new(shared_data.clone()).map_err(|_| AppsActorError::Internal)?;
    let is_update = shared_data.lock().registry.get_by_update_url(url).is_some();
    let data_path = request.shared_data.lock().config.data_path();
    let app = request
        .download_and_apply_pwa(&data_path, url, is_update)
        .map_err(AppsActorError::ServiceError)?;
    request
        .shared_data
        .lock()
        .registry
        .event_broadcaster
        .broadcast_app_installed(&app);

    Ok(())
}

// Read valid application from from_path.
// Install it to data_dir.
// Update shared_data with manifest
pub fn install_package(
    shared_data: &Shared<AppsSharedData>,
    from_path: &Path,
    manifest: &Manifest,
) -> Result<(AppsItem, bool), AppsActorError> {
    let path = shared_data.lock().config.data_path();
    let mut manifest = manifest.clone();

    if !from_path.is_file() {
        return Err(AppsActorError::PackageMissing);
    }
    if !path.is_dir() {
        return Err(AppsActorError::InstallationDirNotFound);
    }
    let app_name = shared_data
        .lock()
        .registry
        .get_unique_name(&manifest.get_name(), manifest.get_origin(), None)
        .map_err(|_| AppsActorError::InvalidAppName)?;

    let download_dir = path.join(format!("downloading/{}", &app_name));
    let download_app = download_dir.join("application.zip");

    fs::create_dir_all(&download_dir)?;
    fs::copy(from_path, download_app)?;

    let is_update = shared_data
        .lock()
        .registry
        .get_first_by_name(&app_name)
        .is_some();
    // Need create appsItem object and add to db to reflect status
    let mut apps_item = AppsItem::default(&app_name, shared_data.lock().registry.get_vhost_port());
    if !manifest.get_version().is_empty() {
        apps_item.set_version(&manifest.get_version());
    }
    apps_item.set_install_state(AppsInstallState::Installing);
    shared_data
        .lock()
        .registry
        .event_broadcaster
        .broadcast_app_installing(&AppsObject::from(&apps_item));

    // This will allow install via appscmd to trigger the dependencies update.
    if let Some(b2g_features) = manifest.get_b2g_features() {
        if let Some(deeplinks) = b2g_features.get_deeplinks() {
            let config_url =
                Url::parse(&deeplinks.config()).map_err(|_| AppsActorError::DeepLinkError)?;
            let config_path = download_dir.join("deeplinks_config");
            let (user_agent, lang) = {
                let registry = &shared_data.lock().registry;
                (registry.get_user_agent(), registry.get_lang())
            };
            let downloader =
                Downloader::new(&user_agent, &lang).map_err(|_| AppsActorError::DeepLinkError)?;
            let (result_recv, _) = downloader.download(&config_url, &config_path, None);
            loop {
                if let Ok(result) = result_recv.recv_timeout(Duration::from_secs(10)) {
                    match result {
                        Ok(info) => match info {
                            DownloaderInfo::Done => {
                                break;
                            }
                            _ => {
                                debug!("Download result: {:?}", info);
                            }
                        },
                        Err(err) => {
                            error!("Download deeplink config error: {:?}", err);
                            return Err(AppsActorError::DeepLinkError);
                        }
                    }
                } else {
                    error!("Download error");
                    return Err(AppsActorError::DeepLinkError);
                }
            }

            match deeplinks.process(&config_url, &config_path, None) {
                Ok(paths) => {
                    apps_item.set_deeplink_paths(Some(paths));
                    manifest.update_deeplinks(&apps_item);
                }
                Err(_) => return Err(AppsActorError::DeepLinkError),
            }
        }
    }

    let mut shared = shared_data.lock();
    shared
        .registry
        .apply_download(&mut apps_item, &download_dir, &manifest, is_update)
        .map_err(|_| AppsActorError::WrongRegistration)?;

    if is_update {
        shared
            .registry
            .event_broadcaster
            .broadcast_app_updated(&AppsObject::from(&apps_item));

        shared.vhost_api.app_updated(&app_name);
    } else {
        shared
            .registry
            .event_broadcaster
            .broadcast_app_installed(&AppsObject::from(&apps_item));

        shared.vhost_api.app_installed(&app_name);
    }

    Ok((apps_item, false))
}

pub fn uninstall(
    shared_data: &Shared<AppsSharedData>,
    manifest_url: &Url,
) -> Result<(), AppsActorError> {
    let mut shared = shared_data.lock();
    let app = shared
        .get_by_manifest_url(manifest_url)
        .map_err(|_| AppsActorError::AppNotFound)?;
    let _ = shared
        .registry
        .uninstall_app(manifest_url)
        .map_err(|_| AppsActorError::WrongRegistration)?;

    shared
        .registry
        .event_broadcaster
        .broadcast_app_uninstalled(&manifest_url);

    shared.vhost_api.app_uninstalled(&app.name);

    Ok(())
}

pub fn launch(
    shared_data: &Shared<AppsSharedData>,
    manifest_url: &Url,
) -> Result<(), AppsActorError> {
    // Ensure that this is a valid manifest url.
    let shared = shared_data.lock();
    let _app = shared
        .get_by_manifest_url(manifest_url)
        .map_err(|_| AppsActorError::AppNotFound)?;

    // Call the bridge to relay the command.
    let bridge = GeckoBridgeService::shared_state();
    bridge.lock().apps_service_on_launch(manifest_url);

    Ok(())
}

pub fn get_all(shared_data: &Shared<AppsSharedData>) -> Result<String, AppsActorError> {
    let shared = shared_data.lock();
    match shared.get_all_apps() {
        Ok(apps) => {
            if apps.is_empty() {
                debug!("Empty application list");
                return Ok("".to_string());
            }
            let apps_str = serde_json::to_string(&apps).map_err(|_| AppsActorError::Internal)?;
            debug!("serialized apps is {}", apps_str);

            Ok(apps_str)
        }
        Err(err) => {
            error!("{:?}", err);
            Err(AppsActorError::Internal)
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
        validate_package(&app_zip).unwrap();
    }
    // ZipPackageNotFound
    {
        let app_zip = format!(
            "{}/test-fixtures/apps-from/missing/application.zip",
            current.display()
        );
        match validate_package(&app_zip) {
            Ok(_) => {
                panic!();
            }
            Err(err) => assert_eq!(
                &format!("{}", err),
                "Io error, No such file or directory (os error 2)"
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
                panic!();
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
                panic!();
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
                    panic!();
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
                panic!();
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
    use crate::apps_registry::AppsRegistry;
    use crate::config;
    use crate::service::AppsService;
    use config::Config;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_dir = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir-install", current.display());
    let test_path = Path::new(&test_dir);

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    if let Err(err) = fs::remove_dir_all(&test_path) {
        println!("test_install_app error: {:?}", err);
    }
    if let Err(err) = fs::create_dir_all(&test_path) {
        println!("test_install_app error: {:?}", err);
    }

    let src_app = current.join("test-fixtures/apps-from/helloworldactor/application.zip");
    println!("src_app: {}", &src_app.display());

    // Test from shared object
    {
        let config = Config::new(
            root_dir,
            test_dir.clone(),
            String::from("uds_path"),
            String::from("test"),
            String::from("updater_socket"),
            true,
        );
        AppsService::init_shared_state(&config);
        let shared_data = AppsService::shared_state();
        {
            let mut shared = shared_data.lock();
            let registry = AppsRegistry::initialize(&config, 4443).unwrap();
            shared.registry = registry;
            println!("shared.apps_objects.len: {}", shared.registry.count());
            assert_eq!(6, shared.registry.count());
        }

        // Install
        let manifest = validate_package(&src_app).unwrap();
        let milisec_before_installing = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if let Ok((apps_item, need_restart)) = install_package(&shared_data, &src_app, &manifest) {
            let app_name = apps_item.get_name();
            let shared = shared_data.lock();
            match shared.registry.get_first_by_name(&app_name) {
                Some(app) => {
                    assert_eq!(true, app.get_install_time() >= milisec_before_installing);
                }
                None => {
                    println!("Installation, failed");
                    panic!();
                }
            }
            assert!(!need_restart);
        } else {
            println!("App installed failed");
            panic!();
        }

        // Re-install
        let manifest = validate_package(&src_app).unwrap();
        let milisec_before_reinstalling = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if let Ok((apps_item, need_restart)) = install_package(&shared_data, &src_app, &manifest) {
            let app_name = apps_item.get_name();
            let shared = shared_data.lock();
            match shared.registry.get_first_by_name(&app_name) {
                Some(app) => {
                    assert_eq!(true, app.get_install_time() >= milisec_before_reinstalling);
                }
                None => {
                    println!("Installation, failed");
                    panic!();
                }
            }
            assert!(!need_restart);
        } else {
            println!("App re-installed failed");
            panic!();
        }
    }

    // Test by reloading data from persisted storage.
    {
        if !test_path.is_dir() {
            println!("Webapp dir does not exist.");
            panic!();
        }

        let shared_data = AppsService::shared_state();
        let manifest_url: Url;
        {
            let shared = shared_data.lock();
            let app_name: String = "helloworldactor".into();
            let apps_item = AppsItem::default(&app_name, shared.registry.get_vhost_port());
            manifest_url = apps_item.get_manifest_url();
            if let Ok(app) = shared.get_by_manifest_url(&manifest_url) {
                assert_eq!(app_name, app.name);
            } else {
                println!("get_by_manifest_url failed.");
                panic!();
            }
        }

        // Uninstall
        if uninstall(&shared_data, &manifest_url).is_ok() {
            let shared = shared_data.lock();
            if shared.get_by_manifest_url(&manifest_url).is_ok() {
                println!("get_by_manifest_url should not ok");
                panic!();
            }
        } else {
            println!("uninstall failed");
            panic!();
        }
    }
}

#[cfg(test)]
fn test_get_all() {
    use crate::apps_registry::AppsRegistry;
    use crate::config;
    use crate::service::AppsService;
    use config::Config;
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let root_path = format!("{}/test-fixtures/webapps", current.display());
    let test_dir = format!("{}/test-fixtures/test-apps-dir-get-all", current.display());

    // This dir is created during the test.
    // Tring to remove it at the beginning to make the test at local easy.
    if let Err(err) = fs::remove_dir_all(Path::new(&test_dir)) {
        println!("test_get_all error: {:?}", err);
    }

    if let Err(err) = fs::create_dir_all(PathBuf::from(test_dir.clone())) {
        println!("test_get_all error: {:?}", err);
    }

    println!("Register from: {}", &root_path);

    println!("test_get_all dir: {}", &test_dir);
    let config = Config::new(
        root_path,
        test_dir,
        String::from("uds_path"),
        String::from("test"),
        String::from("updater_socket"),
        true,
    );
    AppsService::init_shared_state(&config);
    let shared_data = AppsService::shared_state();
    {
        let registry = match AppsRegistry::initialize(&config, 8443) {
            Ok(registry) => registry,
            Err(err) => {
                println!("AppsRegistry::initialize error: {:?}", err);
                return;
            }
        };
        {
            let mut shared = shared_data.lock();
            shared.config = config;
            shared.registry = registry;
            shared.state = AppsServiceState::Running;
        }
        let app_list = get_all(&shared_data).unwrap();
        let expected = r#"[{"name":"apps","install_state":"Installed","manifest_url":"http://apps.localhost:8443/manifest.webmanifest","removable":false,"status":"Enabled","update_manifest_url":null,"update_state":"Idle","update_url":"http://127.0.0.1:8596/apps/apps/manifest.webmanifest","allowed_auto_download":false,"preloaded":true,"progress":0,"origin":"http://apps.localhost:8443"},{"name":"calculator","install_state":"Installed","manifest_url":"http://calculator.localhost:8443/manifest.webmanifest","removable":false,"status":"Enabled","update_manifest_url":null,"update_state":"Idle","update_url":"http://127.0.0.1:8596/apps/calculator/manifest.webmanifest","allowed_auto_download":false,"preloaded":true,"progress":0,"origin":"http://calculator.localhost:8443"},{"name":"system","install_state":"Installed","manifest_url":"http://system.localhost:8443/manifest.webmanifest","removable":false,"status":"Enabled","update_manifest_url":null,"update_state":"Idle","update_url":"https://store.server/system/manifest.webmanifest","allowed_auto_download":false,"preloaded":true,"progress":0,"origin":"http://system.localhost:8443"},{"name":"gallery","install_state":"Installed","manifest_url":"http://gallery.localhost:8443/manifest.webmanifest","removable":true,"status":"Enabled","update_manifest_url":null,"update_state":"Idle","update_url":null,"allowed_auto_download":false,"preloaded":true,"progress":0,"origin":"http://gallery.localhost:8443"},{"name":"launcher","install_state":"Installed","manifest_url":"http://launcher.localhost:8443/manifest.webmanifest","removable":false,"status":"Enabled","update_manifest_url":null,"update_state":"Idle","update_url":null,"allowed_auto_download":false,"preloaded":true,"progress":0,"origin":"http://launcher.localhost:8443"},{"name":"preloadpwa","install_state":"Installed","manifest_url":"http://cached.localhost:8443/preloadpwa/manifest.webmanifest","removable":true,"status":"Enabled","update_manifest_url":null,"update_state":"Idle","update_url":"https://preloadpwa.domain.url/manifest.webmanifest","allowed_auto_download":false,"preloaded":true,"progress":0,"origin":"https://preloadpwa.domain.url"}]"#;

        assert_eq!(app_list, expected);
    }
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

    let req = Request {
        cmd: "launch".into(),
        param: Some("app".into()),
    };
    assert!(validate_request(&req));

    let req = Request {
        cmd: "launch".into(),
        param: Some("".into()),
    };
    assert!(validate_request(&req));
}
