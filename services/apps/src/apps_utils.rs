use crate::generated::common::*;
use crate::manifest::Manifest;
use crate::update_manifest::UpdateManifest;
use http::Uri;
use log::{debug, error};

pub fn compare_manifests(
    update_manifest: &UpdateManifest,
    manifest: &Manifest,
) -> Result<(), AppsServiceError> {
    debug!("compare_manifests for {}", manifest.get_name());
    if update_manifest.get_name() != manifest.get_name() {
        error!("App name do not match");
        return Err(AppsServiceError::InvalidAppName);
    }

    if update_manifest.get_origin() != manifest.get_origin() {
        error!("App origin do not match");
        return Err(AppsServiceError::InvalidOrigin);
    }

    if let (Some(update_manifest_features), Some(manifest_features)) = (
        update_manifest.get_b2g_features(),
        manifest.get_b2g_features(),
    ) {
        match (
            update_manifest_features.get_developer(),
            manifest_features.get_developer(),
        ) {
            (Some(developer1), Some(developer2)) => {
                if developer1.get("name") != developer2.get("name")
                    || developer1.get("url") != developer2.get("url")
                {
                    error!("Developer do not match");
                    return Err(AppsServiceError::InvalidManifest);
                }
            }
            (None, Some(_)) | (Some(_), None) => return Err(AppsServiceError::InvalidManifest),
            (None, None) => { // Don't early return in case we add more checks
            }
        }
    }

    Ok(())
}

pub fn is_absolute_uri(uri_str: &str) -> bool {
    if let Ok(uri) = uri_str.parse::<Uri>() {
        uri.scheme_str().is_some()
    } else {
        false
    }
}

#[test]
fn test_compare_manifest_ok() {
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let manifest_path = format!(
        "{}/test-fixtures/test-appsutils/update_manifest.webmanifest",
        current.display()
    );
    let update_manifest = UpdateManifest::read_from(&manifest_path).unwrap();

    let manifest_path = format!(
        "{}/test-fixtures/test-appsutils/manifest_ok.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();

    assert!(compare_manifests(&update_manifest, &manifest).is_ok());
}

#[test]
fn test_compare_manifest_mismatch() {
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let manifest_path1 = format!(
        "{}/test-fixtures/test-appsutils/update_manifest.webmanifest",
        current.display()
    );
    let update_manifest = UpdateManifest::read_from(&manifest_path1).unwrap();
    // Test name mis-match
    let manifest_path = format!(
        "{}/test-fixtures/test-appsutils/name_mismatch.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();
    assert_ne!(update_manifest.get_name(), manifest.get_name());
    assert_eq!(
        compare_manifests(&update_manifest, &manifest),
        Err(AppsServiceError::InvalidAppName)
    );
    // Test dev mis-match
    let manifest_path = format!(
        "{}/test-fixtures/test-appsutils/dev_mismatch.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path).unwrap();
    assert_eq!(update_manifest.get_name(), manifest.get_name());
    assert_eq!(
        compare_manifests(&update_manifest, &manifest),
        Err(AppsServiceError::InvalidManifest)
    );
}
