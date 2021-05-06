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
    })
}

// Testing need the client to have host names configured properly:
// These names need to resolve to 127.0.0.1:
// valid.local
// unknown.local
// missing-zip.local
#[cfg(test)]
mod test {
    use crate::config::Config;
    use crate::vhost_handler::{vhost, AppData};
    use actix_cors::Cors;
    use actix_rt::System;
    use actix_web::client::{Client, ClientBuilder};
    use actix_web::http::StatusCode;
    use actix_web::HttpServer;
    use actix_web::{web, App};
    use common::traits::Shared;
    use log::info;
    use std::collections::HashMap;
    use std::thread;

    // Starts the server in its own thread.
    pub fn launch_server(config: &Config, port: u16) {
        let config = config.clone();
        let app_data = Shared::adopt(AppData {
            root_path: config.root_path.clone(),
            csp: config.csp.clone(),
            zips: HashMap::new(),
        });

        thread::Builder::new()
            .name("virtual host server".into())
            .spawn(move || {
                let sys = actix_rt::System::new("vhost-server");

                info!(
                    "Starting vhost server from {} on localhost:{}",
                    config.root_path, port
                );

                HttpServer::new(move || {
                    App::new()
                        .data(app_data.clone())
                        .wrap(Cors::default().allow_any_origin().send_wildcard())
                        .route("/{filename:.*}", web::get().to(vhost))
                })
                .disable_signals() // For now, since that's causing us issues with Ctrl-C
                .bind(format!("localhost:{}", port))
                .unwrap()
                .run();

                let _ = sys.run();
            })
            .expect("Failed to start vhost server thread");
    }

    // Launches a https server on the given port, and waits a bit to let it be ready.
    fn start_server(port: u16) {
        let config = Config {
            root_path: "./test-fixtures/".into(),
            csp: "default-src * data: blob:; script-src 'self' http://127.0.0.1 http://shared.localhost; object-src 'none'; style-src 'self' 'unsafe-inline' http://shared.localhost".into(),
        };

        thread::Builder::new()
            .name(format!("vhost server on port {}", port))
            .spawn(move || {
                launch_server(&config, port);
            })
            .expect("Failed to start vhost server");

        thread::sleep(std::time::Duration::from_secs(3));
    }

    // Creates a client configured properly for a language and etag behavior.
    fn http_client(lang: Option<&str>, if_none_match: Option<&str>) -> Client {
        let mut builder = ClientBuilder::new();
        if let Some(lang) = lang {
            builder = builder.header(actix_web::http::header::ACCEPT_LANGUAGE, lang);
        }
        if let Some(if_none_match) = if_none_match {
            builder = builder.header(actix_web::http::header::IF_NONE_MATCH, if_none_match);
        }

        builder.finish()
    }

    // Returns the ETag of the request.
    fn lang_request(
        url: &str,
        expected: StatusCode,
        mime: &str,
        lang: Option<&'static str>,
        if_none_match: Option<&'static str>,
    ) -> Result<String, ()> {
        use actix_web::HttpMessage;
        use log::error;

        let url = url.to_owned();
        let mime = mime.to_owned();

        let fut = async move {
            let url2 = url.clone();
            let url3 = url.clone();
            let response = http_client(lang, if_none_match).get(url).send().await;
            response
                .map_err(move |err| {
                    error!("HTTP Request failed: {}", err);
                    panic!("Failed to retrieve {}", url2.clone());
                })
                .and_then(|response| {
                    assert_eq!(response.status(), expected, "{}", url3);
                    if response.status() == StatusCode::OK {
                        assert_eq!(response.content_type(), mime);
                    }
                    // ETag is not set on responses that are 4xx or 5xx.
                    let etag = match response.headers().get("ETag") {
                        Some(etag) => etag.to_str().unwrap(),
                        None => "",
                    };
                    Ok(etag.to_string())
                })
        };
        System::new("test").block_on(fut)
    }

    fn request(url: &str, expected: StatusCode, mime: &str) -> Result<String, ()> {
        lang_request(url, expected, mime, None, None)
    }

    fn request_if_none_match(
        url: &str,
        expected: StatusCode,
        mime: &str,
        if_none_match: &'static str,
    ) -> Result<String, ()> {
        lang_request(url, expected, mime, None, Some(if_none_match))
    }

    // Return the redirect URL.
    fn redirect_request(url: &str, expected: StatusCode) -> Result<String, ()> {
        use log::error;

        let url = url.to_owned();

        let fut = async move {
            let url2 = url.clone();
            let response = http_client(None, None).get(url).send().await;
            response
                .map_err(move |err| {
                    error!("HTTP Request failed: {}", err);
                    panic!("Failed to retrieve {}", url2.clone());
                })
                .and_then(|response| {
                    assert_eq!(response.status(), expected);
                    let location = match response.headers().get("Location") {
                        Some(location) => location.to_str().unwrap(),
                        None => "",
                    };
                    Ok(location.to_string())
                })
        };
        System::new("test").block_on(fut)
    }

    #[test]
    fn valid_vhost() {
        let _ = env_logger::try_init();

        start_server(7443);

        let _ = request(
            "http://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );

        let _ = lang_request(
            "http://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
            None,
        );

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
            None,
        );

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("fr-FR"),
            None,
        );

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::NOT_FOUND,
            "text/html",
            Some("es-SP"),
            None,
        );

        let _ = lang_request(
            "http://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        );

        let _ = request(
            "http://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        );

        let _ = request(
            "http://valid.localhost:7443/index2.html",
            StatusCode::NOT_FOUND,
            "text/html",
        );

        let _ = request(
            "http://valid.localhost:7443/some/file.txt",
            StatusCode::NOT_FOUND,
            "text/plain",
        );

        let _ = request(
            "http://valid.localhost:7443/manifest.webapp",
            StatusCode::OK,
            "application/json",
        );

        let _ = request(
            "http://valid2.localhost:7443/manifest.webmanifest",
            StatusCode::OK,
            "application/manifest+json",
        );

        let _ = request(
            "http://unknown.localhost:7443/index.html",
            StatusCode::NOT_FOUND,
            "text/html",
        );

        let _ = request(
            "http://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );

        let _ = request(
            "http://missing-zip.localhost:7443/js/main.js",
            StatusCode::OK,
            "application/javascript",
        );

        let _ = lang_request(
            "http://missing-zip.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        );

        let _ = lang_request(
            "http://missing-zip.localhost:7443/resources/file.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        );

        let _ = lang_request(
            "http://valid.localhost:7443/resources/file.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        );

        // Testing etag from file.
        let etag = request(
            "http://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        )
        .unwrap();
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
        .unwrap();
        assert_eq!(
            &etag,
            "W/\"69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9\""
        );

        // Testing etag from zip.
        let etag = request(
            "http://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        )
        .unwrap();
        assert_eq!(&etag, "W/\"2927261257-87\"");

        let etag = request_if_none_match(
            "http://valid.localhost:7443/css/style.css",
            StatusCode::NOT_MODIFIED,
            "text/css",
            "W/\"2927261257-87\"",
        )
        .unwrap();
        assert_eq!(&etag, "W/\"2927261257-87\"");

        // Testing redirect URL.
        let url = redirect_request(
            "http://localhost:7443/redirect/valid/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
            StatusCode::MOVED_PERMANENTLY,
        )
        .unwrap();
        assert_eq!(
            &url,
            "http://valid.localhost:7443/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
        );

        let url = redirect_request(
            "http://localhost:7443/redirect/valid/path/to/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
            StatusCode::MOVED_PERMANENTLY,
        )
        .unwrap();
        assert_eq!(
            &url,
            "http://valid.localhost:7443/path/to/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
        );

        let _url = redirect_request(
            "http://localhost:7443/valid/file.html?state=Authenticator&code=ftAFIdZ5Gaxg-pRbq3iDcV_mQwU2VIUDgJ09GT",
            StatusCode::BAD_REQUEST,
        );
    }
}
