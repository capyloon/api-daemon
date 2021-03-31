// Manages uploads of crash reports to a S3 server.
// This code runs at startup to check if the daemon was killed
// by a crash that left a report to send.
// The last time a crash was uploaded is saved in the $LOG_PATH/last_upload
// file as seconds since epoch.

use crate::global_context::GlobalContext;
use crate::shared_state::SharedStateKind;
#[cfg(target_os = "android")]
use android_utils::{AndroidProperties, PropertyGetter};
use flate2::bufread::GzEncoder;
use flate2::Compression;
use log::{debug, error};
use reqwest::header::CONTENT_TYPE;
use rusty_s3::credentials::Credentials;
use rusty_s3::{Bucket, S3Action};
use serde::Serialize;
use std::fs::{read_dir, remove_file, File};
use std::io::{BufReader, Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

// Throttling interval for uploads, in seconds.
static MIN_UPLOAD_INTERVAL: u64 = 3600;

// Credentials to push crash reports to S3.
static S3_ACCESS_KEY_ID: Option<&'static str> = option_env!("AWS_ACCESS_KEY_ID");
static S3_SECRET_ACCESS_KEY: Option<&'static str> = option_env!("AWS_SECRET_ACCESS_KEY");

static S3_ENDPOINT: &str = "https://s3.amazonaws.com";
static S3_REGION: &str = "us-east-1";
static S3_BUCKET: &str = "daemon-crash-logs";

// Helper to abstract the S3 specifics of uploads.
struct S3Helper {
    bucket: Bucket,
    credentials: Credentials,
}

impl S3Helper {
    fn new() -> Self {
        let bucket = Bucket::new(
            reqwest::Url::parse(S3_ENDPOINT).unwrap(),
            true,
            S3_BUCKET.into(),
            S3_REGION.into(),
        )
        .expect("Failed to create S3 bucket. Check parameters!");
        let credentials = Credentials::new(
            S3_ACCESS_KEY_ID.unwrap().replace('\r', ""),
            S3_SECRET_ACCESS_KEY.unwrap().replace('\r', ""),
        );
        Self {
            bucket,
            credentials,
        }
    }

    // Tries to upload a named resource.
    // Returns Ok if the http request was sent, and notifies of the success status.
    fn upload(
        &self,
        name: &str,
        meta: &ReportMetadata,
        data: Vec<u8>,
    ) -> Result<bool, reqwest::Error> {
        let full_name = format!("{}/{}/{}", meta.target, meta.build_type, name);
        let url = self
            .bucket
            .put_object(Some(&self.credentials), &full_name)
            .sign(Duration::from_secs(24 * 60 * 60)); // One day
        let client = reqwest::blocking::Client::new();
        let content_type = mime_guess::from_path(url.as_str()).first_or_octet_stream();
        let response = client
            .put(url)
            .header(CONTENT_TYPE, content_type.as_ref())
            .body(reqwest::blocking::Body::from(data))
            .send()?;
        Ok(response.status().is_success())
    }
}

// Crash report metadata, gathering information about the build
// and the device.
#[derive(Default, Serialize)]
struct ReportMetadata {
    commit: String,   // The git commit of this daemon's build.
    features: String, // The set of features enabled in this build.
    target: String,   // The build target.
    #[cfg(target_os = "android")]
    build_fingerprint: String, // The ro.system.build.fingerprint property
    build_type: String, // The ro.system.build.type property or "userdebug" for desktop
    #[cfg(target_os = "android")]
    build_version_sdk: String, // The ro.system.build.version.sdk
}

impl ReportMetadata {
    fn new() -> Self {
        Self {
            commit: env!("VERGEN_SHA").into(),
            features: env!("VERGEN_CARGO_FEATURES").into(),
            target: env!("VERGEN_CARGO_TARGET_TRIPLE").into(),
            #[cfg(target_os = "android")]
            build_fingerprint: AndroidProperties::get("ro.system.build.fingerprint", "")
                .unwrap_or_default(),
            #[cfg(target_os = "android")]
            build_type: AndroidProperties::get("ro.system.build.type", "").unwrap_or_default(),
            #[cfg(not(target_os = "android"))]
            build_type: "userdebug".into(),
            #[cfg(target_os = "android")]
            build_version_sdk: AndroidProperties::get("ro.system.build.version.sdk", "")
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub struct CrashUploader {
    path: PathBuf,
    last_upload: SystemTime,
    dev_mode: bool, // A flag used to change behavior when running tests or dev.
}

impl CrashUploader {
    fn last_upload_time(path: &Path) -> Result<SystemTime, Error> {
        let mut file = File::open(&path.join("last_upload"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let secs = contents
            .parse::<u64>()
            .map_err(|_| Error::from(ErrorKind::Other))?;
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
    }

    fn save_upload_time(&self) -> Result<(), Error> {
        let elapsed = SystemTime::UNIX_EPOCH
            .elapsed()
            .map_err(|_| Error::from(ErrorKind::Other))?;

        let mut file = File::create(&self.path.join("last_upload"))?;
        file.write_fmt(format_args!("{}", elapsed.as_secs()))?;

        Ok(())
    }

    pub fn new(path: &str) -> Self {
        // Default to epoch if we don't have a valid value, since it should
        // only happen at first crash when there is no saved upload time.
        let last_upload =
            CrashUploader::last_upload_time(&Path::new(path)).unwrap_or(SystemTime::UNIX_EPOCH);

        #[cfg(target_os = "android")]
        let dev_mode =
            AndroidProperties::get("ro.system.build.type", "").unwrap_or_default() == "userdebug";

        #[cfg(not(target_os = "android"))]
        let dev_mode = true;

        Self {
            path: PathBuf::from(path),
            last_upload,
            dev_mode,
        }
    }

    // Check if can upload crash reports. This is true if all these conditions are met:
    // - user consent was granted.
    // - we don't need to throttle uploads.
    // - S3 credentials are available.
    // Note: this code only runs at daemon startup so we don't need to listen for settings change.
    pub fn can_upload(&self, global_context: &GlobalContext) -> bool {
        if S3_ACCESS_KEY_ID.is_none() || S3_SECRET_ACCESS_KEY.is_none() {
            error!(
                "Invalid S3 authentication data: `{:?}` `{:?}`",
                S3_ACCESS_KEY_ID, S3_SECRET_ACCESS_KEY
            );
            return false;
        }

        // Verify that the user gave consent by checking the value of the "eventlogger.telemetry.consent" setting.
        let service_state = global_context.service_state();
        let lock = service_state.lock();
        let res = if let Some(SharedStateKind::SettingsService(data)) =
            lock.get(&"SettingsManager".to_string())
        {
            match data.lock().db.get("eventlogger.telemetry.consent") {
                Ok(value) => *value == serde_json::Value::Bool(true),
                _ => {
                    error!("No `eventlogger.telemetry.consent` setting found, denying crash report upload.");
                    false
                }
            }
        } else {
            error!("Failed to access SettingsService.");
            false
        };

        // When in dev mode, don't block on delay.
        if self.dev_mode {
            return res;
        }

        if let Ok(last_upload_duration) = self.last_upload.elapsed() {
            if last_upload_duration.as_secs() < MIN_UPLOAD_INTERVAL {
                error!(
                    "Last crash report was uploaded {}s ago, but we need to wait at least {}s",
                    last_upload_duration.as_secs(),
                    MIN_UPLOAD_INTERVAL
                );
                return false;
            }
        } else {
            // Stay on the safe side if any clock issue happens.
            error!("Can't compute crash report upload time difference");
            return false;
        }

        res
    }

    // Collects the list of reports from the reports directory.
    fn list_reports(&self) -> Vec<PathBuf> {
        if !self.path.is_dir() {
            error!(
                "Failed to list crash reports, {} is not a directory.",
                self.path.display()
            );
            return vec![];
        }

        let dmp_ext = Some(std::ffi::OsStr::new("dmp"));
        let mut res = vec![];

        // We keep going as much as possible in case of error instead of
        // returning early to clean up as much as possible.
        if let Ok(reader) = read_dir(&self.path) {
            for entry in reader {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension() == dmp_ext {
                        res.push(path);
                    }
                }
            }
        }

        res
    }

    // Remove all the crash reports. This happens either when we can't send them or after sending them
    // in order to not use too much disk space.
    pub fn wipe_reports(&self) {
        if self.dev_mode {
            debug!("In dev mode, not wiping out crash reports");
            return;
        }

        for path in self.list_reports() {
            debug!("Will delete crash report {:?}", path);
            let _ = remove_file(path);
        }
    }

    // Tries to upload the reports available, and wipe them after upload.
    pub fn upload_reports(&self) {
        let reports = self.list_reports();
        if reports.is_empty() {
            return;
        }

        let uploader = self.clone();
        thread::Builder::new()
            .name("crash report uploader".into())
            .spawn(move || {
                // Shared helper for multiple uploads.
                let s3 = S3Helper::new();
                let meta = ReportMetadata::new();

                for path in reports {
                    debug!("About to upload crash report {:?}", path);

                    // Read and compress the minidump.
                    match compress_file(&path) {
                        Ok(data) => {
                            // Get the file name. It's fine to unwrap because we already
                            // checked that the file has a .dmp extension.
                            let name =
                                format!("{}.gz", path.file_name().unwrap().to_string_lossy());
                            // Now upload it...
                            match s3.upload(&name, &meta, data) {
                                Ok(true) => debug!("Success uploading {}", name),
                                Ok(false) => error!("Server error uploading {}", name),
                                Err(err) => error!("Upload failure for {} : {}", name, err),
                            }

                            // Upload the metadata.
                            let name =
                                format!("{}.json", path.file_stem().unwrap().to_string_lossy());
                            match s3.upload(
                                &name,
                                &meta,
                                serde_json::to_vec(&meta).unwrap_or_default(),
                            ) {
                                Ok(true) => debug!("Success uploading {}", name),
                                Ok(false) => error!("Server error uploading {}", name),
                                Err(err) => error!("Upload failure for {} : {}", name, err),
                            }

                            // And remove it, even if we failed to upload.
                            if !uploader.dev_mode {
                                let _ = remove_file(path);
                            }
                        }
                        Err(err) => error!("Failed to read {:?}: {}", path, err),
                    }
                }

                let _ = uploader.save_upload_time();
            })
            .expect("Failed to create crash report uploader thread");
    }
}

// Returns the gzip compression of a file.
fn compress_file(path: &Path) -> Result<Vec<u8>, Error> {
    let file = File::open(path)?;
    let mut gz = GzEncoder::new(BufReader::new(file), Compression::best());
    let mut result = Vec::new();
    gz.read_to_end(&mut result)?;
    Ok(result)
}
