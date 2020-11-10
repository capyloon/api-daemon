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
    pub name: String,
    pub version: String, //TODO: Version,
    pub package_path: String,
    pub packaged_size: u64,
    pub size: u64,
    #[serde(default = "UpdateManifest::default_dependencies")]
    pub dependencies: HashMap<String, String>, //VersionReq>,
    pub r#type: String,
    pub b2g_features: Option<B2GFeatures>,
}

impl UpdateManifest {
    fn default_dependencies() -> HashMap<String, String> {
        HashMap::<String, String>::new()
    }

    pub fn read_from<P: AsRef<Path>>(manifest_file: P) -> Result<Self, AppsMgmtError> {
        let file = std::fs::File::open(manifest_file)?;
        serde_json::from_reader(std::io::BufReader::new(file)).map_err(|err| err.into())
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
                manifest.package_path,
                "https://seinlin.org/apps/packages/sample/sample-signed.zip"
            );
            assert_eq!(manifest.size, 10022);
            assert_eq!(manifest.packaged_size, 12345);
            assert_eq!(manifest.r#type, "web");
        }
        Err(err) => {
            error!("Error: {:?}", err);
            assert!(false, "Failed to read {}", manifest_path);
        }
    }
}
