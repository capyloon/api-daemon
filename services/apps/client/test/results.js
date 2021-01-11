var calculator_expected = {"name":"calculator","installState":0,"manifestUrl":"http://calculator.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateUrl":"https://store.server/calculator/manifest.webmanifest","allowedAutoDownload":false};

// removable is true per test fixtures.
var gallery_expected = {"name":"gallery","installState":0,"manifestUrl":"http://gallery.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateUrl":"https://store.server/gallery/manifest.webmanifest","allowedAutoDownload":false};

var system_expected = {"name":"system","installState":0,"manifestUrl":"http://system.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateUrl":"https://store.server/system/manifest.webmanifest","allowedAutoDownload":false};

// updateUrl is empty on purpose
var launcher_expected = {"name":"launcher","installState":0,"manifestUrl":"http://launcher.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateUrl":"","allowedAutoDownload":false};

function install_expected(installState) {
  return {"name":"ciautotest","installState":installState,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","status":0,"updateState":0,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};
}

function update_expected(updateState) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":updateState,"updateUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","status":0,"updateState":0,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};
}

function status_expected(status) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":status,"updateState":0,"updateUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","status":0,"updateState":0,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};
}

function launcher_status_expected(status) {
    return {"name":"launcher","installState":0,"manifestUrl":"http://launcher.localhost:8081/manifest.webmanifest","removable":false,"status":status,"updateState":0,"updateUrl":"","allowedAutoDownload":false};
}

var get_all_expected0 = [ calculator_expected, gallery_expected, system_expected, launcher_expected ];

var get_all_expected1 = [ calculator_expected, gallery_expected, system_expected, launcher_expected, install_expected(0) ];

var download_failed_response_expected = {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","status":0,"updateState":0,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};

function install_pwa_expected(installState){
  return {"name":"hellopwa","installState":installState,"manifestUrl":"http://cached.localhost:8081/hellopwa/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateUrl":"https://testpwa.github.io/manifest.webmanifest","allowedAutoDownload":false};
}

var get_all_expected2 = [ calculator_expected, gallery_expected, system_expected, launcher_expected, install_pwa_expected(0) ];

var expected_sha1 = "93 37 60 C8 86 16 F0 A6 68 D8 9C 1C 2F E6 F6 7B 62 57 06 0F";

