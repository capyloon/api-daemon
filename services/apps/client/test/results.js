var apps_expected = {"name": "apps","installState": 0,"manifestUrl": "http://apps.localhost:8081/manifest.webmanifest","removable": false,"status": 0,"updateManifestUrl": "","updateState": 0,"updateUrl": "http://127.0.0.1:8596/apps/apps/manifest.webmanifest","allowedAutoDownload": false,"preloaded": true,"progress": 0,"origin":"http://apps.localhost:8081"};

var pwa_expected = {"name":"preloadpwa","installState":0,"manifestUrl":"http://cached.localhost:8081/preloadpwa/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"","updateState":0,"updateUrl":"https://preloadpwa.domain.url/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"https://preloadpwa.domain.url"};

var calculator_expected = {"name":"calculator","installState":0,"manifestUrl":"http://calculator.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"","updateUrl":"http://127.0.0.1:8596/apps/calculator/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://calculator.localhost:8081"};
var calculator_update_expected = {"name":"calculator","installState":0,"manifestUrl":"http://calculator.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/calculator/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/calculator/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://calculator.localhost:8081"};

// removable is true per test fixtures.
var gallery_expected = {"name":"gallery","installState":0,"manifestUrl":"http://gallery.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"","updateState":0,"updateUrl":"","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://gallery.localhost:8081"};

var system_expected = {"name":"system","installState":0,"manifestUrl":"http://system.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"","updateUrl":"https://store.server/system/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://system.localhost:8081"};

// updateUrl is empty on purpose
var launcher_expected = {"name":"launcher","installState":0,"manifestUrl":"http://launcher.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"","updateUrl":"","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://launcher.localhost:8081"};

function install_expected(installState, progress=0) {
  return {"name":"ciautotest","installState":installState,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":progress,"origin":"http://ciautotest.localhost:8081"};
}

function update_expected(updateState, allowedAutoDownload=false) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":updateState,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":allowedAutoDownload,"preloaded":false,"progress":0,"origin":"http://ciautotest.localhost:8081"};
}

function update_expected_pre_installed(updateState, allowedAutoDownload=false) {
    return {"name":"calculator","installState":0,"manifestUrl":"http://calculator.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":updateState,"updateManifestUrl":"http://cached.localhost:8081/calculator/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/calculator/manifest.webmanifest","allowedAutoDownload":allowedAutoDownload,"preloaded":true,"progress":0,"origin":"http://calculator.localhost:8081"};
}

function status_expected(status) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":status,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":0,"origin":"http://ciautotest.localhost:8081"};
}

function launcher_status_expected(status) {
    return {"name":"launcher","installState":0,"manifestUrl":"http://launcher.localhost:8081/manifest.webmanifest","removable":false,"status":status,"updateState":0,"updateManifestUrl":"","updateUrl":"","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://launcher.localhost:8081"};
}

var get_all_expected0 = [ apps_expected, pwa_expected, calculator_expected, gallery_expected, system_expected, launcher_expected ];

var get_all_expected1 = [ apps_expected, pwa_expected, calculator_expected, gallery_expected, system_expected, launcher_expected, install_expected(0) ];

var download_failed_response_expected = {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":0,"origin":"http://ciautotest.localhost:8081"};

function install_pwa_expected(installState, status=0) {
  return {"name":"hellopwa","installState":installState,"manifestUrl":"http://cached.localhost:8081/hellopwa/manifest.webmanifest","removable":true,"status":status,"updateManifestUrl":"http://cached.localhost:8081/hellopwa/update.webmanifest","updateState":0,"updateUrl":"http://127.0.0.1:8596/apps/pwa/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":0,"origin":"http://127.0.0.1:8596"};
}

function relative_pwa_expected(installState) {
  return {"name":"relativepwa","installState":installState,"manifestUrl":"http://cached.localhost:8081/relativepwa/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"http://cached.localhost:8081/relativepwa/update.webmanifest","updateState":0,"updateUrl":"http://127.0.0.1:8596/apps/pwa/relative.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":0,"origin":"http://127.0.0.1:8596"};
}

function same_origin_pwa_expected(installState) {
  return {"name":"sameoriginpwa","installState":installState,"manifestUrl":"http://cached.localhost:8081/sameoriginpwa/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"http://cached.localhost:8081/sameoriginpwa/update.webmanifest","updateState":0,"updateUrl":"http://127.0.0.1:8596/apps/pwa/same-origin.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":0,"origin":"http://127.0.0.1:8596"};
}

function update_pwa_expected(updateState, allowedAutoDownload=false) {
  return {"name":"hellopwa","installState":0,"manifestUrl":"http://cached.localhost:8081/hellopwa/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"http://cached.localhost:8081/hellopwa/update.webmanifest","updateState":updateState,"updateUrl":"http://127.0.0.1:8596/apps/pwa/manifest.webmanifest","allowedAutoDownload":allowedAutoDownload,"preloaded":false,"progress":0,"origin":"http://127.0.0.1:8596"};
}

// reason and updateUrl is accurate anytime
// some times apps Object is not properly constructed.
// That's because it fails to get manifest.
var download_canceled_event = {"appsObject":{"name":"ciautotest","installState":lib_apps.AppsInstallState.INSTALLING,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateState":0,"updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false,"progress":0,"origin":"http://ciautotest.localhost:8081"},"reason":1};

var new_gallery_expected = {"name":"newgallery","installState":0,"manifestUrl":"http://newgallery.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"","updateState":0,"updateUrl":"","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"http://newgallery.localhost:8081"};

var new_pwa_expected = {"name":"newpreloadpwa","installState":0,"manifestUrl":"http://cached.localhost:8081/newpreloadpwa/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"","updateState":0,"updateUrl":"https://newpreloadpwa.domain.url/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true,"progress":0,"origin":"https://newpreloadpwa.domain.url"};

var get_all_expected2 = [ apps_expected, calculator_update_expected, new_pwa_expected, new_gallery_expected, system_expected, launcher_expected, install_expected(0), install_pwa_expected(0), relative_pwa_expected(0), same_origin_pwa_expected(0) ];

var get_all_expected3 = [ apps_expected, calculator_update_expected, new_gallery_expected, system_expected, launcher_expected, install_pwa_expected(0), relative_pwa_expected(0), same_origin_pwa_expected(0) ];

var get_all_expected4 = [ apps_expected, calculator_update_expected, new_gallery_expected, system_expected, launcher_expected, install_expected(0), install_pwa_expected(0), relative_pwa_expected(0), same_origin_pwa_expected(0) ];

var expected_sha1 = "B2 95 1A FD 74 7F 40 B7 E9 D2 E6 37 A3 5D 12 F3 B8 5B 0E 4A";

var config_expected = {"enabled":true,"connType":0,"delay":86400};
