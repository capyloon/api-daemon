use crate::manifest::Manifest;
use crate::update_manifest::UpdateManifest;

use http::Uri;

pub fn compare_manifests(update_manifest: &UpdateManifest, manifest: &Manifest) -> bool {
    if update_manifest.get_name() != manifest.get_name() {
        return false;
    }
    let maybe_update_manifest_features = update_manifest.get_b2g_features();
    let maybe_manifest_features = manifest.get_b2g_features();
    if let (Some(update_manifest_features), Some(manifest_features)) =
        (maybe_update_manifest_features, maybe_manifest_features)
    {
        let maybe_dev1 = update_manifest_features.get_developer();
        let maybe_dev2 = manifest_features.get_developer();

        match (maybe_dev1, maybe_dev2) {
            (Some(developer1), Some(developer2)) => {
                if assert_json_diff::assert_json_eq_no_panic(&developer1, &developer2).is_err() {
                    return false;
                }
            }
            (None, Some(_)) | (Some(_), None) => return false,
            (None, None) => { // Don't early return in case we add more checks
            }
        }
    }

    true
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
        "{}/test-fixtures/test-appsutils/update_manifest_ok.webmanifest",
        current.display()
    );
    let update_manifest = UpdateManifest::read_from(&manifest_path).unwrap();

    let manifest_path2 = format!(
        "{}/test-fixtures/test-appsutils/manifest_ok.webmanifest",
        current.display()
    );
    let manifest = Manifest::read_from(&manifest_path2).unwrap();

    assert!(compare_manifests(&update_manifest, &manifest));
}

#[test]
fn test_compare_manifest_name_mismatch() {
    use std::env;

    let _ = env_logger::try_init();

    // Init apps from test-fixtures/webapps and verify in test-apps-dir.
    let current = env::current_dir().unwrap();
    let manifest_path1 = format!(
        "{}/test-fixtures/test-appsutils/name_mismatch1.webmanifest",
        current.display()
    );
    let update_manifest = UpdateManifest::read_from(&manifest_path1).unwrap();
    let manifest_path2 = format!(
        "{}/test-fixtures/test-appsutils/name_mismatch2.webmanifest",
        current.display()
    );
    let manifest2 = Manifest::read_from(&manifest_path2).unwrap();
    assert_ne!(update_manifest.get_name(), manifest2.get_name());
    assert!(!compare_manifests(&update_manifest, &manifest2));
}
