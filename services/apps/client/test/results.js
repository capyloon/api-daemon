var calculator_expected = {"name":"calculator","installState":0,"manifestUrl":"http://calculator.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"","updateUrl":"http://127.0.0.1:8596/apps/calculator/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true};

// removable is true per test fixtures.
var gallery_expected = {"name":"gallery","installState":0,"manifestUrl":"http://gallery.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"","updateState":0,"updateUrl":"","allowedAutoDownload":false,"preloaded":true};

var system_expected = {"name":"system","installState":0,"manifestUrl":"http://system.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"","updateUrl":"https://store.server/system/manifest.webmanifest","allowedAutoDownload":false,"preloaded":true};

// updateUrl is empty on purpose
var launcher_expected = {"name":"launcher","installState":0,"manifestUrl":"http://launcher.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":0,"updateManifestUrl":"","updateUrl":"","allowedAutoDownload":false,"preloaded":true};

function install_expected(installState) {
  return {"name":"ciautotest","installState":installState,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false};
}

function update_expected(updateState, allowedAutoDownload=false) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":updateState,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":allowedAutoDownload,"preloaded":false};
}

function update_expected_pre_installed(updateState, allowedAutoDownload=false) {
    return {"name":"calculator","installState":0,"manifestUrl":"http://calculator.localhost:8081/manifest.webmanifest","removable":false,"status":0,"updateState":updateState,"updateManifestUrl":"","updateUrl":"http://127.0.0.1:8596/apps/calculator/manifest.webmanifest","allowedAutoDownload":allowedAutoDownload,"preloaded":true};
}

function status_expected(status) {
  return {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":status,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false};
}

function launcher_status_expected(status) {
    return {"name":"launcher","installState":0,"manifestUrl":"http://launcher.localhost:8081/manifest.webmanifest","removable":false,"status":status,"updateState":0,"updateManifestUrl":"","updateUrl":"","allowedAutoDownload":false,"preloaded":true};
}

var get_all_expected0 = [ calculator_expected, gallery_expected, system_expected, launcher_expected ];

var get_all_expected1 = [ calculator_expected, gallery_expected, system_expected, launcher_expected, install_expected(0) ];

var download_failed_response_expected = {"name":"ciautotest","installState":0,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateState":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false};

function install_pwa_expected(installState) {
  return {"name":"hellopwa","installState":installState,"manifestUrl":"http://cached.localhost:8081/hellopwa/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"http://cached.localhost:8081/hellopwa/update.webmanifest","updateState":0,"updateUrl":"https://testpwa.github.io/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false};
}

// reason and updateUrl is accurate anytime
// some times apps Object is not properly constructed.
// That's because it fails to get manifest.
var download_canceled_event = {"appsObject":{"name":"ciautotest","installState":lib_apps.AppsInstallState.INSTALLING,"manifestUrl":"http://ciautotest.localhost:8081/manifest.webmanifest","removable":true,"status":0,"updateManifestUrl":"http://cached.localhost:8081/ciautotest/update.webmanifest","updateState":0,"updateUrl":"http://127.0.0.1:8596/apps/ciautotest/manifest.webmanifest","allowedAutoDownload":false,"preloaded":false},"reason":1}
var get_all_expected2 = [ calculator_expected, gallery_expected, system_expected, launcher_expected, install_pwa_expected(0) ];
var get_all_expected3 = [ calculator_expected, gallery_expected, system_expected, launcher_expected, install_pwa_expected(0), install_expected(0) ];

var expected_sha1 = "B2 95 1A FD 74 7F 40 B7 E9 D2 E6 37 A3 5D 12 F3 B8 5B 0E 4A";
// token_type: "hawk", scope: "u|core:cruds sc#apps:rs sc#metrics:c payment#products:rs payment#purchases:crud simcustm#pack:s simcustm#packfile:r payment#transactions:cr payment#prices:s payment#options:s", expires_in: 604800, kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=", mac_algorithm: "hmac-sha-256" }
//["Hawk id=\"FGFYvY+/4XwTYIX9nVi+sXj5tPA=\"", "ts=\"1611717940\"", "nonce=\"SrnmiS6u9dckTg==\"", "mac=\"gVH14LHIxSTD/Oq7+MsFCpxHzafWRDSEvXlGFnpQAzM=\"", "hash=\"\""]
var token = { keyId: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", macKey: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=" };

class TokenProvider extends lib_apps.TokenProviderBase {
    constructor(service, session) {
        super(service.id, session);
    }

    display() {
        return "TokenProvider";
    }

    getToken(tokenType) {
        console.log('TokenProvider getToken() is called');

        return Promise.resolve({
            keyId: token.keyId,
            macKey: token.macKey,
            tokenType: tokenType,
        });
    }
}

var config_expected = {"enabled":true,"connType":0,"delay":86400};
