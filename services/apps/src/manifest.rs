// Representation of a manifest, as used for install and sideloading.

use crate::apps_item::AppsItem;
use crate::apps_registry::AppsMgmtError;
use crate::apps_request::AppsRequest;
use crate::apps_utils;
use crate::deeplinks::DeepLinks;
use crate::generated::common::*;

use common::JsonValue;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Manifest name missing")]
    NameMissing,
    #[error("Manifest missing")]
    ManifestMissing,
    #[error("Manifest wrong format")]
    ManifestWrongFormat,
    #[error("Cannot be absolute url")]
    AbsoluteUrl,
    #[error("Json Error {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct B2GFeatures {
    #[serde(default = "String::new")]
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    deeplinks: Option<DeepLinks>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<HashMap<String, String>>, // A list of hashMap<package_name, package_version>
    #[serde(skip_serializing_if = "Option::is_none")]
    origin: Option<String>,
    #[serde(default = "default_as_false")]
    from_legacy: bool,
}

pub trait ExtendUrl {
    fn same_scope(&self, match_url: &Url) -> bool;
}

impl ExtendUrl for Url {
    fn same_scope(&self, match_url: &Url) -> bool {
        match self.make_relative(match_url) {
            Some(path) => {
                if !path.starts_with("..") {
                    return true;
                }
            }
            None => return false,
        }
        false
    }
}

impl From<&Manifest> for JsonValue {
    fn from(v: &Manifest) -> Self {
        JsonValue::from(serde_json::to_value(v).unwrap_or_else(|_| json!({})))
    }
}

fn default_as_false() -> bool {
    false
}

impl B2GFeatures {
    pub fn get_locales(&self) -> Option<Value> {
        self.locales.clone()
    }

    pub fn get_developer(&self) -> Option<Value> {
        self.developer.clone()
    }

    pub fn get_deeplinks(&self) -> Option<DeepLinks> {
        self.deeplinks.clone()
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

    pub fn get_default_locale(&self) -> String {
        self.default_locale.clone()
    }

    pub fn get_permissions(&self) -> Option<Value> {
        self.permissions.clone()
    }

    pub fn get_origin(&self) -> Option<String> {
        self.origin.clone()
    }

    pub fn set_deeplinks(&mut self, deeplinks: Option<DeepLinks>) {
        self.deeplinks = deeplinks;
    }

    pub fn is_from_legacy(&self) -> bool {
        self.from_legacy
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Icon {
    src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sizes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    purpose: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Shortcut {
    #[serde(default = "String::new")]
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default = "String::new")]
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icons: Option<Vec<Icon>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Manifest {
    name: String,
    #[serde(default = "default_as_start_url")]
    start_url: String,
    icons: Option<Value>, // to backward compatible with icons object
    b2g_features: Option<B2GFeatures>,
    #[serde(default = "String::new")]
    display: String,
    #[serde(default = "String::new")]
    short_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(default = "String::new")]
    dir: String,
    #[serde(default = "String::new")]
    lang: String,
    #[serde(default = "String::new")]
    orientation: String,
    #[serde(default = "String::new")]
    theme_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    shortcuts: Option<Vec<Shortcut>>,
}

fn default_as_start_url() -> String {
    "/".into()
}

impl Icon {
    pub fn set_src(&mut self, src: &str) {
        self.src = src.to_string();
    }

    pub fn get_src(&self) -> String {
        self.src.clone()
    }

    pub fn get_sizes(&self) -> Option<String> {
        self.sizes.clone()
    }

    pub fn get_type(&self) -> Option<String> {
        self.r#type.clone()
    }

    // https://www.w3.org/TR/appmanifest/#purpose-member
    // Icon purpose list: "monochrome", "maskable", "any"
    // If purpose doesn't exist, it will be used as any(default).
    // If an icon contains multiple purpose, it could be used for any of those purpose.
    // If none of the stated purpose are recognized, the icon is totally ignored.
    pub fn process_purpose(&mut self) -> bool {
        if let Some(purpose) = &self.purpose {
            if purpose.is_empty() {
                self.purpose = Some("any".into());
                return true;
            }
            let mut processed_purpose = vec![];
            for value in purpose.split_whitespace() {
                if ["monochrome", "maskable", "any"].contains(&value) {
                    processed_purpose.push(value);
                }
            }
            if processed_purpose.is_empty() {
                return false;
            }
            self.purpose = Some(processed_purpose.join(" "));
        } else {
            self.purpose = Some("any".into());
        }
        true
    }

    pub fn process(
        &mut self,
        request: &AppsRequest,
        update_url_base: &Url,
        manifest_url_base: &Url,
        download_dir: &Path,
    ) -> Result<(), AppsServiceError> {
        // If none of the stated purpose are recognized, the icon is totally ignored.
        if !self.process_purpose() {
            return Ok(());
        }
        request.download_icon(self, update_url_base, manifest_url_base, download_dir)
    }
}

impl Shortcut {
    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub fn set_icons(&mut self, icons: Option<Vec<Icon>>) {
        self.icons = icons;
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_icons(&self) -> Option<Vec<Icon>> {
        self.icons.clone()
    }

    // https://www.w3.org/TR/appmanifest/#dfn-process-the-shortcuts-member
    pub fn process(&mut self, manifest_url: &Url) -> bool {
        if self.name.is_empty() {
            return false;
        }
        if self.url.is_empty() {
            return false;
        }
        if let Ok(url) = manifest_url.join(&self.url) {
            if !url.same_scope(manifest_url) {
                return false;
            }
            self.set_url(url.as_str());
        } else {
            return false;
        }
        true
    }
}

impl Manifest {
    #[cfg(test)]
    pub fn new(name: &str, start_url: &str, b2g_features: Option<B2GFeatures>) -> Self {
        let url = if start_url.is_empty() {
            default_as_start_url()
        } else {
            start_url.to_owned()
        };
        Manifest {
            name: name.to_string(),
            start_url: url,
            b2g_features,
            ..Default::default()
        }
    }

    pub fn check_validity(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::NameMissing);
        }

        // We verify the properties in b2g_features
        if let Some(b2g_features) = self.get_b2g_features() {
            if let Some(activities_json) = b2g_features.get_activities() {
                if let Some(activities) = activities_json.as_object() {
                    for activity in activities.values() {
                        if let Some(href) = activity.get("href").unwrap_or(&json!(null)).as_str() {
                            if apps_utils::is_absolute_uri(href) {
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
                                    if apps_utils::is_absolute_uri(href) {
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

    pub fn update_deeplinks(&mut self, apps_item: &AppsItem) {
        if let Some(mut b2g_features) = self.get_b2g_features() {
            if let Some(mut deeplinks) = b2g_features.get_deeplinks() {
                deeplinks.set_paths(apps_item.get_deeplink_paths());
                b2g_features.set_deeplinks(Some(deeplinks));
                self.set_b2g_features(Some(b2g_features));
            }
        }
    }

    pub fn set_b2g_features(&mut self, b2g_features: Option<B2GFeatures>) {
        self.b2g_features = b2g_features;
    }

    pub fn set_icons(&mut self, icons: Value) {
        self.icons = Some(icons);
    }

    pub fn set_start_url(&mut self, url: &str) {
        self.start_url = url.to_string();
    }

    pub fn set_scope(&mut self, scope: &str) {
        self.scope = Some(scope.to_string());
    }

    pub fn set_shortcuts(&mut self, shortcuts: Option<Vec<Shortcut>>) {
        self.shortcuts = shortcuts;
    }

    pub fn get_shortcuts(&self) -> Option<Vec<Shortcut>> {
        self.shortcuts.clone()
    }

    pub fn get_origin(&self) -> Option<String> {
        let b2g_features = self.get_b2g_features()?;
        b2g_features.get_origin()
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

    pub fn get_start_url(&self) -> String {
        self.start_url.clone()
    }

    pub fn get_scope(&self) -> Option<String> {
        self.scope.clone()
    }

    pub fn get_b2g_features(&self) -> Option<B2GFeatures> {
        self.b2g_features.clone()
    }

    pub fn get_icons(&self) -> Option<Value> {
        self.icons.clone()
    }

    // Process the scope member of the app manifest.
    //   In:
    //     The app manifest url
    //   Return:
    //     The result of the processing
    pub fn process_scope(&mut self, base_url: &Url) -> Result<(), AppsServiceError> {
        debug!("process_scope with base_url {}", base_url);
        let start_url = base_url
            .join(&self.get_start_url())
            .map_err(|_| AppsServiceError::InvalidScope)?;

        let scope = match self.get_scope() {
            Some(manifest_scope) => {
                if manifest_scope.is_empty() {
                    return Err(AppsServiceError::InvalidScope);
                }
                // If the manifest json["scope"] is not empty.,
                // set the scope with manifest URL as base.
                base_url
                    .join(&manifest_scope)
                    .map_err(|_| AppsServiceError::InvalidScope)?
            }
            None => {
                // If scope is missing in manifest use default scope.
                // the restult of parsing '.' with start_url as base URL.
                start_url
                    .join(".")
                    .map_err(|_| AppsServiceError::InvalidScope)?
            }
        };

        // The app scope and start url need to be the same scope.
        if !scope.same_scope(&start_url) {
            return Err(AppsServiceError::InvalidScope);
        }
        self.set_scope(scope.as_str());
        Ok(())
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LegacyManifest {
    name: String,
    icons: Option<Value>,
    #[serde(default = "default_as_start_url")]
    start_url: String,
    #[serde(default = "String::new")]
    launch_path: String,
    #[serde(default = "String::new")]
    display: String,
    #[serde(default = "String::new")]
    short_name: String,
    #[serde(default = "String::new")]
    orientation: String,
    #[serde(default = "String::new")]
    theme_color: String,
    #[serde(default = "String::new")]
    default_locale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    locales: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    developer: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activities: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    messages: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<HashMap<String, String>>,
}

impl From<LegacyManifest> for Manifest {
    fn from(s: LegacyManifest) -> Self {
        let b2g_features = B2GFeatures {
            default_locale: s.default_locale.clone(),
            locales: s.locales.clone(),
            developer: s.developer.clone(),
            permissions: s.permissions.clone(),
            activities: s.activities.clone(),
            messages: s.messages.clone(),
            version: s.version.clone(),
            dependencies: s.dependencies.clone(),
            from_legacy: true,
            ..Default::default()
        };
        let start_url = match s.launch_path.is_empty() {
            true => s.start_url.clone(),
            false => s.launch_path.clone(),
        };
        Manifest {
            name: s.name.clone(),
            start_url,
            icons: s.icons.clone(),
            b2g_features: Some(b2g_features),
            display: s.display.clone(),
            lang: s.default_locale.clone(),
            short_name: s.short_name.clone(),
            orientation: s.orientation.clone(),
            theme_color: s.theme_color,
            ..Default::default()
        }
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
    fn test_check_validity_href_ok() {
        use std::env;

        let _ = env_logger::try_init();

        // Init apps from test-fixtures/webapps and verify in test-apps-dir.
        let current = env::current_dir().unwrap();
        let manifest_path = format!(
            "{}/test-fixtures/test-appsutils/href_ok.webmanifest",
            current.display()
        );
        let manifest = Manifest::read_from(&manifest_path).unwrap();

        assert!(manifest.check_validity().is_ok());
    }

    #[test]
    fn test_check_validity_href_nok() {
        use std::env;

        let _ = env_logger::try_init();

        // Init apps from test-fixtures/webapps and verify in test-apps-dir.
        let current = env::current_dir().unwrap();
        let manifest_path = format!(
            "{}/test-fixtures/test-appsutils/href_nok.webmanifest",
            current.display()
        );
        let manifest = Manifest::read_from(&manifest_path).unwrap();

        assert!(manifest.check_validity().is_err());
    }
}

#[test]
fn test_same_scope() {
    let base_url = Url::parse("https://domain.com/foo/").unwrap();

    assert!(base_url.same_scope(&Url::parse("https://domain.com/foo/index.html").unwrap()));
    assert!(base_url.same_scope(&Url::parse("https://domain.com/foo/bar/index.html").unwrap()));
    assert!(base_url.same_scope(&Url::parse("https://domain.com/foo/bar/../index.html").unwrap()));
    assert!(base_url.same_scope(&Url::parse("https://domain.com/xyz/../foo/index.html").unwrap()));
    assert!(
        base_url.same_scope(&Url::parse("https://domain.com/xyz/../foo/bar/index.html").unwrap())
    );

    assert!(!base_url.same_scope(&Url::parse("https://domain.com/foo/../index.html").unwrap()));
    assert!(!base_url.same_scope(&Url::parse("https://different.com/foo/bar/index.html").unwrap()));
    assert!(!base_url.same_scope(&Url::parse("https://domain.com/index.html").unwrap()));
    assert!(!base_url.same_scope(&Url::parse("https://domain.com/xyz/index.html").unwrap()));
    assert!(!base_url.same_scope(&Url::parse("https://domain.com/bar/foo/index.html").unwrap()));
}

#[test]
fn test_icon_purpose() {
    use std::env;

    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/test-purpose/valid.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();
    match manifest.get_icons() {
        Some(icons_value) => {
            let mut icons: Vec<Icon> =
                serde_json::from_value(icons_value).unwrap_or_else(|_| Vec::new());
            for icon in &mut icons {
                assert!(icon.process_purpose());
            }
        }
        None => panic!(),
    }

    let manifest_path = format!(
        "{}/test-fixtures/test-purpose/invalid.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();
    match manifest.get_icons() {
        Some(icons_value) => {
            let mut icons: Vec<Icon> =
                serde_json::from_value(icons_value).unwrap_or_else(|_| Vec::new());
            for icon in &mut icons {
                assert!(!icon.process_purpose());
            }
        }
        None => panic!(),
    }
}

#[test]
fn test_manifest_shortcuts() {
    use std::env;

    let manifest_url = Url::parse("https://domain.com/manifest.webmanifest").unwrap();
    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/test-shortcuts/valid.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();
    match manifest.get_shortcuts() {
        Some(mut shortcuts) => {
            for shortcut in &mut shortcuts {
                assert!(shortcut.process(&manifest_url));
                assert!([
                    "https://domain.com/subscriptions?sort=desc",
                    "https://domain.com/play-later"
                ]
                .contains(&shortcut.get_url().as_str()));
            }
        }
        None => panic!(),
    }

    let manifest_path = format!(
        "{}/test-fixtures/test-shortcuts/invalid.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();
    match manifest.get_shortcuts() {
        Some(mut shortcuts) => {
            for shortcut in &mut shortcuts {
                assert!(!shortcut.process(&manifest_url));
            }
        }
        None => panic!(),
    }
}

#[test]
fn test_manifest_scopes() {
    use std::env;

    let manifest_url = Url::parse("https://domain.com/manifest.webmanifest").unwrap();
    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/test-scope/hasscope.webmanifest",
        current.display()
    );
    let mut manifest = Manifest::read_from(&manifest_path).unwrap();
    assert_eq!(manifest.process_scope(&manifest_url), Ok(()));
    assert_eq!(manifest.get_scope(), Some("https://domain.com/foo/".into()));

    let manifest_path = format!(
        "{}/test-fixtures/test-scope/noscope.webmanifest",
        current.display()
    );
    let mut manifest = Manifest::read_from(&manifest_path).unwrap();
    assert_eq!(manifest.process_scope(&manifest_url), Ok(()));
    assert_eq!(manifest.get_scope(), Some("https://domain.com/foo/".into()));

    let manifest_path = format!(
        "{}/test-fixtures/test-scope/noscope-2.webmanifest",
        current.display()
    );
    let mut manifest = Manifest::read_from(&manifest_path).unwrap();
    assert_eq!(manifest.process_scope(&manifest_url), Ok(()));
    assert_eq!(manifest.get_scope(), Some("https://domain.com/bar/".into()));

    let manifest_path = format!(
        "{}/test-fixtures/test-scope/invalid.webmanifest",
        current.display()
    );
    let mut manifest = Manifest::read_from(&manifest_path).unwrap();
    assert_eq!(
        manifest.process_scope(&manifest_url),
        Err(AppsServiceError::InvalidScope)
    );
}
