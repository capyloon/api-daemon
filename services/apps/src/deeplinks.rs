use crate::apps_registry::AppsMgmtError;
use crate::generated::common::AppsServiceError;
use crate::manifest::ExtendUrl;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::fs::File;
use std::path::Path;
use url::Url;

///
/// The Deeplinks object in the app manifest.
///   config: A config file hosted on the remote server with the same origin to
///           the path to trigger the deeplinks open.
///   paths: The apps service will read the config file, resolve each path
///          with the config url and set set to this paths object.
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeepLinks {
    #[serde(default = "String::new")]
    config: String,
    #[serde(default = "DeepLinks::default_paths")]
    paths: Option<Value>,
}

impl DeepLinks {
    fn default_paths() -> Option<Value> {
        None
    }

    pub fn config(&self) -> String {
        self.config.clone()
    }

    pub fn paths(&self) -> Option<Value> {
        self.paths.clone()
    }

    pub fn set_paths(&mut self, paths: Option<Value>) {
        self.paths = paths;
    }

    ///
    /// Validate the Deeplinks object and return the URLs list for deeplink match.
    ///   config_url: A config file hosted on the remote server with the same origin
    ///               to the path to trigger the deeplinks action.
    ///   config_path: The apps service will read the config file, resolve each path
    ///                with the config url and set set to this paths object.
    ///   update_url (Optinal): App's update_url. Mandatory for install from store.
    ///
    pub fn process(
        &self,
        config_url: &Url,
        config_path: &Path,
        update_url: Option<&Url>,
    ) -> Result<Value, AppsServiceError> {
        let config =
            Self::read_from(config_path).map_err(|_| AppsServiceError::InvalidDeeplinks)?;

        // The config must contain at least one entry.
        let apps_config = config.apps();
        if apps_config.is_empty() {
            error!("Empty config.");
            return Err(AppsServiceError::InvalidDeeplinks);
        }

        let mut paths = Vec::new();
        if update_url.is_none() {
            // App installed by appscmd do not have manifest_url,
            // use the first entry.
            paths = apps_config[0].paths();
        } else if let Some(update_url) = update_url {
            let url_str = update_url.as_str();
            debug!("process deeplink for: {}", url_str);
            for app in &apps_config {
                if app.manifest_url() == url_str {
                    paths = app.paths();
                    break;
                }
            }
        }

        if paths.is_empty() {
            error!("Invalid paths for deeplinks.");
            return Err(AppsServiceError::InvalidDeeplinks);
        }

        let mut joined_paths: Vec<String> = Vec::new();
        for path in &paths {
            debug!("Check path: {}", path);
            let path_url = config_url
                .join(path)
                .map_err(|_| AppsServiceError::InvalidDeeplinks)?;

            // To prevent hijacking the top level scope without the ownership.
            if !config_url.same_scope(&path_url) {
                error!(
                    "The scope is not the same as config url: {}.",
                    path_url.as_str()
                );
                return Err(AppsServiceError::InvalidDeeplinks);
            }

            joined_paths.push(path_url.as_str().to_string());
        }
        Ok(json!(joined_paths))
    }

    pub fn read_from<P: AsRef<Path>>(config_file: P) -> Result<DeepLinksConfig, AppsMgmtError> {
        let file = File::open(config_file)?;
        serde_json::from_reader(std::io::BufReader::new(file)).map_err(|err| err.into())
    }
}

///
/// Deelinks config file include a json object that define the path(s)
/// to trigger the deepplinks open for one of more apps.
///
/// ```{
///     "deeplinks": "",
///     "apps": [
///         {
///             "manifest_url": "https://api.kaiostech.com/apps/manifest/kzLiaFQOTlGk8DJePIQA",
///             "paths": [ "/news/", "/headlines/"]
///         },
///         {
///           ...
///         }
///     ]
/// }```
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeepLinksConfig {
    deeplinks: String,
    apps: Vec<AppsDeepLinks>,
}

impl DeepLinksConfig {
    pub fn apps(&self) -> Vec<AppsDeepLinks> {
        self.apps.clone()
    }
}

///
/// The object that defines deeplink config for an App.
///   manifest_url: The app manifest URL is used as identifier and
///       defined by the app store or PWA app provider.
///       For example:
///       https://api.kaiostech.com/apps/manifest/kzLiaFQOTlGk8DJePIQA
///       https://www.google.com/maps/preview/pwa/kaios/manifest.webapp
///   paths: The array of paths.
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppsDeepLinks {
    #[serde(default = "String::new")]
    manifest_url: String,
    #[serde(default = "Vec::new")]
    paths: Vec<String>,
}

impl AppsDeepLinks {
    pub fn manifest_url(&self) -> String {
        self.manifest_url.clone()
    }
    pub fn paths(&self) -> Vec<String> {
        self.paths.clone()
    }
}

#[test]
fn test_deeplinks_process() {
    use crate::apps_storage::AppsStorage;

    // Run tests from test-fixtures/deeplinks dir.
    let current = std::env::current_dir().unwrap();
    let test_dir = format!("{}/test-fixtures/deeplinks", current.display());
    let test_path = Path::new(&test_dir);
    let manifest = AppsStorage::load_manifest(&test_path).unwrap();
    let deeplink = match manifest.get_b2g_features() {
        Some(b2g_features) => b2g_features.get_deeplinks().unwrap(),
        None => panic!("No deeplink object in the manifest."),
    };
    let config_url = Url::parse(&deeplink.config()).unwrap();
    let config_path = test_path.join("app-links-config");

    // A dummy Url with appsid for tests.
    let update_url = Url::parse("https://store-domain.com/apps/path/kzLiaFQOTlGk8DJePIQA").unwrap();

    let empty_path = test_path.join("empty-links-config");
    assert_eq!(
        deeplink.process(&config_url, &empty_path, Some(&update_url)),
        Err(AppsServiceError::InvalidDeeplinks)
    );

    let incomplete_path = test_path.join("empty-links-config");
    assert_eq!(
        deeplink.process(&config_url, &incomplete_path, Some(&update_url)),
        Err(AppsServiceError::InvalidDeeplinks)
    );

    let wrong_scope_path = test_path.join("wrong-scope-config");
    assert_eq!(
        deeplink.process(&config_url, &wrong_scope_path, Some(&update_url)),
        Err(AppsServiceError::InvalidDeeplinks)
    );

    let wrong_scope_path = test_path.join("wrong-scope-config2");
    assert_eq!(
        deeplink.process(&config_url, &wrong_scope_path, Some(&update_url)),
        Err(AppsServiceError::InvalidDeeplinks)
    );

    let config_test_url = Url::parse("http://service-domain.com/test/app-links-config").unwrap();
    assert_eq!(
        deeplink.process(&config_test_url, &config_path, Some(&update_url)),
        Err(AppsServiceError::InvalidDeeplinks)
    );

    let wrong_update_url = Url::parse("https://store-domain.com/apps/path/wrongAppId").unwrap();
    assert_eq!(
        deeplink.process(&config_url, &config_path, Some(&wrong_update_url)),
        Err(AppsServiceError::InvalidDeeplinks)
    );

    // Test with update_url
    if let Ok(paths) = deeplink.process(&config_url, &config_path, Some(&update_url)) {
        assert_eq!(
            paths,
            json!([
                "http://service-domain.com/apps/",
                "http://service-domain.com/apps/service/"
            ])
        );
    } else {
        panic!("Unexpected result.");
    }

    // Test without update_url
    if let Ok(paths) = deeplink.process(&config_url, &config_path, None) {
        assert_eq!(
            paths,
            json!([
                "http://service-domain.com/apps/",
                "http://service-domain.com/apps/service/"
            ])
        );
    } else {
        panic!("Unexpected result.");
    }
}
