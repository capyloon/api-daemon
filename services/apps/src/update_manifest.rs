// Representation of an update manifest.

use crate::apps_registry::AppsMgmtError;
use crate::manifest::B2GFeatures;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppsPermission {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateManifest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    packaged_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(default = "UpdateManifest::default_dependencies")]
    dependencies: HashMap<String, String>, // A list of hashMap<package_name, package_version>
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    b2g_features: Option<B2GFeatures>,
}

impl UpdateManifest {
    fn default_dependencies() -> HashMap<String, String> {
        HashMap::<String, String>::new()
    }

    pub fn read_from<P: AsRef<Path>>(manifest_file: P) -> Result<Self, AppsMgmtError> {
        let file = std::fs::File::open(manifest_file)?;
        serde_json::from_reader(std::io::BufReader::new(file)).map_err(|err| err.into())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_version(&self) -> String {
        self.version.clone().unwrap_or_else(|| "".into())
    }

    pub fn get_package_path(&self) -> String {
        self.package_path.clone().unwrap_or_else(|| "".into())
    }

    pub fn get_packaged_size(&self) -> u64 {
        self.packaged_size.clone().unwrap_or(0)
    }

    pub fn get_size(&self) -> u64 {
        self.size.clone().unwrap_or(0)
    }

    pub fn get_type(&self) -> String {
        self.r#type.clone().unwrap_or_else(|| "".into())
    }

    pub fn get_b2g_features(&self) -> Option<B2GFeatures> {
        self.b2g_features.clone()
    }
}

#[test]
fn test_read_manifest() {
    use log::error;
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/sample_update_manifest.webmanifest",
        current.display()
    );

    match UpdateManifest::read_from(&manifest_path) {
        Ok(manifest) => {
            assert_eq!(manifest.name, "Sample1");
            assert_eq!(
                manifest.get_package_path(),
                "https://seinlin.org/apps/packages/sample/sample-signed.zip"
            );
            assert_eq!(manifest.get_size(), 10022);
            assert_eq!(manifest.get_packaged_size(), 12345);
            assert_eq!(manifest.get_type(), "web");

            let packages = manifest.get_dependencies();
            assert_eq!(packages.len(), 3);
        }
        Err(err) => {
            error!("Error: {:?}", err);
            assert!(false, "Failed to read {}", manifest_path);
        }
    }
}
