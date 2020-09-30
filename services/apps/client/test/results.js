var calculator_expected = {"name":"calculator","installState":0,"manifestUrl":"https://calculator.local/manifest.webapp","status":0,"updateState":0,"updateUrl":"https://store.server/calculator/manifest.webapp","allowedAutoDownload":false};

var gallery_expected = {"name":"gallery","installState":0,"manifestUrl":"https://gallery.local/manifest.webapp","status":0,"updateState":0,"updateUrl":"https://store.server/gallery/manifest.webapp","allowedAutoDownload":false};

var system_expected = {"name":"system","installState":0,"manifestUrl":"https://system.local/manifest.webapp","status":0,"updateState":0,"updateUrl":"https://store.server/system/manifest.webapp","allowedAutoDownload":false};

// updateUrl is empty on purpose
var launcher_expected = {"name":"launcher","installState":0,"manifestUrl":"https://launcher.local/manifest.webapp","status":0,"updateState":0,"updateUrl":"","allowedAutoDownload":false};

function install_expected(installState) {
  return {"name":"ciautotest","installState":installState,"manifestUrl":"https://ciautotest.local/manifest.webapp","status":0,"updateState":0,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};
}

function update_expected(updateState) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"https://ciautotest.local/manifest.webapp","status":0,"updateState":updateState,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};
}

var get_all_expected0 = [ calculator_expected, gallery_expected, system_expected, launcher_expected ];

var get_all_expected1 = [ calculator_expected, gallery_expected, system_expected, launcher_expected, install_expected(0) ];

var get_all_expected2 = [ calculator_expected, gallery_expected, system_expected, launcher_expected ];

var download_failed_response_expected = {"name":"ciautotest","installState":0,"manifestUrl":"https://ciautotest.local/manifest.webapp","status":0,"updateState":0,"updateUrl":"http://127.0.0.1:8081/tests/fixtures/packaged_app_manifest.json","allowedAutoDownload":false};