/// A simple vhost http server.
use common::traits::Shared;
use std::collections::HashMap;

pub mod config;
pub mod etag;
pub mod vhost_handler;

use config::Config;
use vhost_handler::AppData;

// Returns the Actix App used to served vhost requests.
pub fn vhost_data(config: &Config) -> Shared<AppData> {
    let config = config.clone();
    Shared::adopt(AppData {
        root_path: config.root_path.clone(),
        csp: config.csp,
        zips: HashMap::new(),
        mappings: HashMap::new(),
    })
}

// Testing need the client to have host names configured properly:
// These names need to resolve to 127.0.0.1:
// valid.local
// unknown.local
// missing-zip.local
#[cfg(test)]
mod test {
    use crate::{AppData, Config};
    use actix_cors::Cors;
    use actix_web::http::{header, StatusCode, Uri};
    use actix_web::{test, web, App};
    use common::traits::Shared;
    use std::collections::HashMap;

    fn get_data() -> Shared<AppData> {
        let config = Config {
            root_path: "./test-fixtures/".into(),
            csp: "default-src * data: blob:; script-src 'self' http://127.0.0.1 http://shared.localhost; object-src 'none'; style-src 'self' 'unsafe-inline' http://shared.localhost".into(),
        };
        let mut mappings = HashMap::new();
        mappings.insert("mapped".into(), "valid".into());

        let app_data = Shared::adopt(AppData {
            root_path: config.root_path.clone(),
            csp: config.csp.clone(),
            zips: HashMap::new(),
            mappings,
        });

        app_data
    }

    // Returns the (ETag, Location) of the request.
    async fn lang_request(
        url: &'static str,
        expected_status: StatusCode,
        expected_mime: &'static str,
        lang: Option<&'static str>,
        if_none_match: Option<&'static str>,
    ) -> (String, String) {
        let mut app = test::init_service(
            App::new()
                .wrap(Cors::default().allow_any_origin().send_wildcard())
                .service(
                    web::scope("")
                        .app_data(web::Data::new(get_data()))
                        .route("/{filename:.*}", web::get().to(crate::vhost_handler::vhost)),
                ),
        )
        .await;

        let uri = Uri::from_static(url);

        let mut req = test::TestRequest::get()
            .uri(uri.path_and_query().unwrap().as_str())
            .insert_header((header::HOST, uri.authority().unwrap().as_str()));

        if let Some(lang) = lang {
            req = req.insert_header((header::ACCEPT_LANGUAGE, lang));
        }

        if let Some(if_none_match) = if_none_match {
            req = req.insert_header((header::IF_NONE_MATCH, if_none_match));
        }

        let resp = test::call_service(&mut app, req.to_request()).await;

        assert_eq!(resp.status(), expected_status);
        let headers = resp.headers();
        if expected_status == StatusCode::OK && !expected_mime.is_empty() {
            let mime_type = headers.get(&header::CONTENT_TYPE).unwrap();
            assert_eq!(mime_type, expected_mime);
        }

        let etag = headers
            .get(&header::ETAG)
            .map(|v| v.to_str().unwrap_or_default().to_owned())
            .unwrap_or_else(|| "".to_owned());
        let location = headers
            .get(&header::LOCATION)
            .map(|v| v.to_str().unwrap_or_default().to_owned())
            .unwrap_or_else(|| "".to_owned());

        (etag, location)
    }

    // Returns the ETag
    async fn request_if_none_match(
        url: &'static str,
        expected: StatusCode,
        mime: &'static str,
        if_none_match: &'static str,
    ) -> String {
        lang_request(url, expected, mime, None, Some(if_none_match))
            .await
            .0
    }

    // Returns the ETag
    async fn request(
        url: &'static str,
        expected_status: StatusCode,
        expected_mime: &'static str,
    ) -> String {
        lang_request(url, expected_status, expected_mime, None, None)
            .await
            .0
    }

    // Returns the Location
    async fn redirect_request(url: &'static str, expected_status: StatusCode) -> String {
        lang_request(url, expected_status, "", None, None).await.1
    }

    #[actix_rt::test]
    async fn simple_requests() {
        let _ = env_logger::init();

        request(
            "http://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        )
        .await;

        request(
            "http://missing-zip.localhost:7443/with_param?v=1234",
            StatusCode::OK,
            "application/octet-stream",
        )
        .await;

        request(
            "http://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        )
        .await;

        request(
            "http://valid.localhost:7443/index2.html",
            StatusCode::NOT_FOUND,
            "text/html",
        )
        .await;

        request(
            "http://valid.localhost:7443/some/file.txt",
            StatusCode::NOT_FOUND,
            "text/plain",
        )
        .await;

        request(
            "http://valid.localhost:7443/manifest.webapp",
            StatusCode::OK,
            "application/json",
        )
        .await;

        request(
            "http://valid2.localhost:7443/manifest.webmanifest",
            StatusCode::OK,
            "application/manifest+json",
        )
        .await;

        request(
            "http://unknown.localhost:7443/index.html",
            StatusCode::NOT_FOUND,
            "text/html",
        )
        .await;

        request(
            "http://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        )
        .await;

        request(
            "http://missing-zip.localhost:7443/js/main.js",
            StatusCode::OK,
            "application/javascript",
        )
        .await;
    }

    #[actix_rt::test]
    async fn lang_requests() {
        let _ = lang_request(
            "http://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
            None,
        )
        .await;

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
            None,
        )
        .await;

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("fr-FR"),
            None,
        )
        .await;

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::NOT_FOUND,
            "text/html",
            Some("es-SP"),
            None,
        )
        .await;

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        )
        .await;
    }

    #[actix_rt::test]
    async fn etag_requests() {
        // Testing etag from file.
        let etag = request(
            "http://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        )
        .await;
        assert_eq!(
            &etag,
            "W/\"69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9\""
        );

        let etag = request_if_none_match(
            "http://missing-zip.localhost:7443/index.html",
            StatusCode::NOT_MODIFIED,
            "text/html",
            "W/\"69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9\"",
        )
        .await;
        assert_eq!(
            &etag,
            "W/\"69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9\""
        );

        // Testing etag from zip.
        let etag = request(
            "jttp://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        )
        .await;
        assert_eq!(&etag, "W/\"2927261257-87\"");

        let etag = request_if_none_match(
            "http://valid.localhost:7443/css/style.css",
            StatusCode::NOT_MODIFIED,
            "text/css",
            "W/\"2927261257-87\"",
        )
        .await;
        assert_eq!(&etag, "W/\"2927261257-87\"");
    }

    #[actix_rt::test]
    async fn redirects() {
        // Testing redirect URL.
        let url = redirect_request(
            "http://localhost:7443/redirect/valid/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
            StatusCode::MOVED_PERMANENTLY,
        )
        .await;
        assert_eq!(
            &url,
            "http://valid.localhost:7443/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
        );

        let url = redirect_request(
            "http://localhost:7443/redirect/valid/path/to/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
            StatusCode::MOVED_PERMANENTLY,
        )
        .await;
        assert_eq!(
            &url,
            "http://valid.localhost:7443/path/to/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
        );

        let _url = redirect_request(
            "http://localhost:7443/valid/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
            StatusCode::BAD_REQUEST,
        );

        let _ = request(
            "http://mapped.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );
    }
}
