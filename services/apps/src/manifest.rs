// Representation of a manifest, as used for install and sideloading.

use crate::apps_registry::AppsMgmtError;
use crate::apps_utils;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Manifest name missing")]
    NameMissing,
    #[error("Manifest launch_path missing")]
    LaunchPathMissing,
    #[error("Manifest missing")]
    ManifestMissing,
    #[error("Manifest wrong format")]
    ManifestWrongFormat,
    #[error("Cannot be absolute url")]
    AbsoluteUrl,
    #[error("Json Error {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct B2GFeatures {
    #[serde(default = "String::new")]
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    developer: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Value>,
    #[serde(default = "String::new")]
    default_locale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    locales: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activities: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputs: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirects: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serviceworker: Option<Value>,
    #[serde(default = "default_as_false")]
    core: bool,
    #[serde(default = "default_as_false")]
    cursor: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    focus_color: Option<String>,
    #[serde(default = "B2GFeatures::default_hashmap")]
    dependencies: HashMap<String, String>, // A list of hashMap<package_name, package_version>
}

fn default_as_false() -> bool {
    false
}

impl B2GFeatures {
    fn default_hashmap() -> HashMap<String, String> {
        HashMap::new()
    }
    pub fn get_locales(&self) -> Option<Value> {
        self.locales.clone()
    }

    pub fn get_developer(&self) -> Option<Value> {
        self.developer.clone()
    }

    pub fn get_activities(&self) -> Option<Value> {
        self.activities.clone()
    }

    pub fn get_messages(&self) -> Option<Value> {
        self.messages.clone()
    }

    pub fn get_version(&self) -> Option<String> {
        self.version.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Icons {
    src: String,
    sizes: String,
    r#type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Manifest {
    name: String,
    #[serde(default = "String::new")]
    launch_path: String,
    #[serde(default = "default_as_start_url")]
    start_url: String,
    icons: Option<Value>, // to backward compatible with icons object
    b2g_features: Option<B2GFeatures>,
    #[serde(default = "String::new")]
    display: String,
    #[serde(default = "String::new")]
    short_name: String,
    #[serde(default = "String::new")]
    scope: String,
    #[serde(default = "String::new")]
    dir: String,
    #[serde(default = "String::new")]
    lang: String,
    #[serde(default = "String::new")]
    orientation: String,
    #[serde(default = "String::new")]
    theme_color: String,
}

fn default_as_start_url() -> String {
    "/index.html".into()
}

impl Icons {
    pub fn set_src(&mut self, src: &str) {
        self.src = src.to_string();
    }

    pub fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn get_sizes(&self) -> String {
        self.sizes.clone()
    }

    pub fn get_type(&self) -> Option<String> {
        self.r#type.clone()
    }
}

impl Manifest {
    pub fn new(name: &str, launch_path: &str, b2g_features: Option<B2GFeatures>) -> Self {
        Manifest {
            name: name.to_string(),
            launch_path: launch_path.to_string(),
            b2g_features,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::NameMissing);
        }

        let launch_path = self.get_launch_path();
        let start_url = self.get_start_url();
        if launch_path.is_empty() && start_url.is_empty() {
            return Err(ManifestError::LaunchPathMissing);
        }

        // We verify the properties in b2g_features
        if let Some(b2g_features) = self.get_b2g_features() {
            if let Some(activities_json) = b2g_features.get_activities() {
                if let Some(activities) = activities_json.as_object() {
                    for activity in activities.values() {
                        if let Some(href) = activity.get("href").unwrap_or(&json!(null)).as_str() {
                            if apps_utils::is_absolute_uri(&href) {
                                return Err(ManifestError::AbsoluteUrl);
                            }
                        }
                    }
                }
            }

            // |messages| is an array of items, where each item is either a string or
            // a {name: href} object.
            if let Some(message_json) = b2g_features.get_messages() {
                if let Some(messages) = message_json.as_array() {
                    for message in messages {
                        if let Some(message_obj) = message.as_object() {
                            for value in message_obj.values() {
                                if let Some(href) = value.as_str() {
                                    if apps_utils::is_absolute_uri(&href) {
                                        return Err(ManifestError::AbsoluteUrl);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    return Err(ManifestError::ManifestWrongFormat);
                }
            }
        }

        Ok(())
    }

    pub fn set_icons(&mut self, icons: Value) {
        self.icons = Some(icons);
    }

    pub fn set_start_url(&mut self, url: &str) {
        self.start_url = url.to_string();
    }

    pub fn get_version(&self) -> String {
        if let Some(b2g_features) = self.get_b2g_features() {
            if let Some(version) = b2g_features.get_version() {
                debug!("version from b2g_features {}", version);
                return version;
            }
        }

        String::new()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }

    pub fn get_launch_path(&self) -> String {
        self.launch_path.clone()
    }

    pub fn get_start_url(&self) -> String {
        self.start_url.clone()
    }

    pub fn get_b2g_features(&self) -> Option<B2GFeatures> {
        self.b2g_features.clone()
    }

    pub fn get_icons(&self) -> Option<Value> {
        self.icons.clone()
    }

    pub fn read_from<P: AsRef<Path>>(manifest_file: P) -> Result<Manifest, AppsMgmtError> {
        let file = File::open(manifest_file)?;
        serde_json::from_reader(std::io::BufReader::new(file)).map_err(|err| err.into())
    }

    pub fn write_to<P: AsRef<Path>>(
        manifest_file: P,
        manifest: &Manifest,
    ) -> Result<(), AppsMgmtError> {
        let file = File::create(manifest_file)?;
        serde_json::to_writer(file, manifest).map_err(|err| err.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::Manifest;
    #[test]
    fn test_read_manifest() {
        use crate::manifest::Manifest;
        use log::error;
        use std::env;

        let _ = env_logger::try_init();

        // Init apps from test-fixtures/webapps and verify in test-apps-dir.
        let current = env::current_dir().unwrap();
        let manifest_path = format!(
            "{}/test-fixtures/sample_app_manifest_1.webmanifest",
            current.display()
        );

        match Manifest::read_from(&manifest_path) {
            Ok(manifest) => {
                assert_eq!(manifest.name, "CIAutoTest");
                assert!(manifest.get_b2g_features().is_some());
            }
            Err(err) => {
                error!("Error: {:?}", err);
                assert!(false, "Failed to read {}", manifest_path);
            }
        }

        let manifest_path = format!(
            "{}/test-fixtures/sample_app_manifest_2.webmanifest",
            current.display()
        );

        match Manifest::read_from(&manifest_path) {
            Ok(manifest) => {
                assert_eq!(manifest.name, "CIAutoTest");
                assert!(manifest.get_b2g_features().is_none());
            }
            Err(err) => {
                error!("Error: {:?}", err);
                assert!(false, "Failed to read {}", manifest_path);
            }
        }

        let manifest_path = format!(
            "{}/test-fixtures/sample_app_manifest_3.webmanifest",
            current.display()
        );

        match Manifest::read_from(&manifest_path) {
            Ok(manifest) => {
                assert_eq!(manifest.name, "CIAutoTest");
                assert!(manifest.get_b2g_features().is_some());
            }
            Err(err) => {
                error!("Error: {:?}", err);
                assert!(false, "Failed to read {}", manifest_path);
            }
        }
    }

    #[test]
    fn test_is_valid_href_ok() {
        use std::env;

        let _ = env_logger::try_init();

        // Init apps from test-fixtures/webapps and verify in test-apps-dir.
        let current = env::current_dir().unwrap();
        let manifest_path = format!(
            "{}/test-fixtures/test-appsutils/href_ok.webmanifest",
            current.display()
        );
        let manifest = Manifest::read_from(&manifest_path).unwrap();

        assert!(manifest.is_valid().is_ok());
    }

    #[test]
    fn test_is_valid_href_nok() {
        use std::env;

        let _ = env_logger::try_init();

        // Init apps from test-fixtures/webapps and verify in test-apps-dir.
        let current = env::current_dir().unwrap();
        let manifest_path = format!(
            "{}/test-fixtures/test-appsutils/href_nok.webmanifest",
            current.display()
        );
        let manifest = Manifest::read_from(&manifest_path).unwrap();

        assert!(manifest.is_valid().is_err());
    }
}
