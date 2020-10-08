/// A simple vhost http server.
use crate::config::VhostApi;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use common::traits::Shared;
use log::info;
use rustls::{
    internal::pemfile::{certs, pkcs8_private_keys},
    NoClientAuth, ServerConfig as RustlsServerConfig,
};
use std::collections::HashMap;
use std::thread;

pub mod config;
mod etag;
mod vhost_handler;

use vhost_handler::{vhost, AppData};

fn get_tls_server_config(config: &config::Config) -> Result<RustlsServerConfig, ()> {
    use std::fs::File;
    use std::io::BufReader;

    let mut server_config = RustlsServerConfig::new(NoClientAuth::new());
    // Only http/1.1 until we figure out why http/2.0 fails.
    server_config.set_protocols(&[b"http/1.1".to_vec()]);

    let cert_file = &mut BufReader::new(File::open(&config.cert_path).map_err(|_| ())?);
    let key_file = &mut BufReader::new(File::open(&config.key_path).map_err(|_| ())?);
    let cert_chain = certs(cert_file)?;
    let mut keys = pkcs8_private_keys(key_file)?;
    server_config
        .set_single_cert(cert_chain, keys.remove(0))
        .map_err(|_| ())?;
    Ok(server_config)
}

// Starts the server in its own thread.
pub fn start_server(config: &config::Config) -> config::VhostApi {
    let config = config.clone();
    let app_data = Shared::adopt(AppData {
        root_path: config.root_path.clone(),
        csp: config.csp.clone(),
        zips: HashMap::new(),
    });

    let vhost_api = VhostApi::new(app_data.clone());

    thread::Builder::new()
        .name("virtual host server".into())
        .spawn(move || {
            let sys = actix_rt::System::new("vhost-server");

            info!(
                "Starting vhost server from {} on localhost:{}",
                config.root_path, config.port
            );

            let tls_config = match get_tls_server_config(&config) {
                Ok(config) => config,
                Err(_) => return,
            };

            HttpServer::new(move || {
                App::new()
                    .data(app_data.clone())
                    .wrap(Logger::new("\"%r\" %{Host}i %s %b %D")) // Custom log to display the vhost
                    .wrap(Cors::new().send_wildcard().finish())
                    .route("/{filename:.*}", web::get().to(vhost))
            })
            .disable_signals() // For now, since that's causing us issues with Ctrl-C
            .bind_rustls(format!("localhost:{}", config.port), tls_config)
            .unwrap()
            .run();

            let _ = sys.run();
        })
        .expect("Failed to start vhost server thread");

    vhost_api
}

// Testing need the client to have host names configured properly:
// These names need to resolve to 127.0.0.1:
// valid.local
// unknown.local
// missing-zip.local
#[cfg(test)]
mod test {
    use crate::config;
    use actix_rt::System;
    use actix_web::client::{Client, ClientBuilder, Connector};
    use actix_web::http::StatusCode;
    use rustls::*;
    use std::thread;

    pub struct NoCertificateVerification {}

    impl rustls::ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _roots: &rustls::RootCertStore,
            _presented_certs: &[rustls::Certificate],
            _dns_name: webpki::DNSNameRef<'_>,
            _ocsp: &[u8],
        ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
            Ok(rustls::ServerCertVerified::assertion())
        }
    }

    // Launches a https server on the given port, and waits a bit to let it be ready.
    fn start_server(port: u16) {
        let config = config::Config {
            port,
            root_path: "./test-fixtures/".into(),
            cert_path: "./cert.pem".into(),
            key_path: "./key.pem".into(),
            csp: "default-src * data: blob:; script-src 'self' http://127.0.0.1 https://shared.local; object-src 'none'; style-src 'self' 'unsafe-inline' https://lighttheme.local https://darktheme.local https://shared.local".into(),
        };

        thread::Builder::new()
            .name(format!("vhost server on port {}", port))
            .spawn(move || {
                crate::start_server(&config);
            })
            .expect("Failed to start vhost server");

        thread::sleep(std::time::Duration::from_secs(3));
    }

    // Creates a client configured to ignore errors when checking the
    // self-signed certificate.
    fn ssl_client(lang: Option<&str>, if_none_match: Option<&str>) -> Client {
        use std::sync::Arc;

        let mut config = ClientConfig::new();
        let protos = vec![b"http/1.1".to_vec()];
        config.set_protocols(&protos);
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
        let connector = Connector::new().rustls(Arc::new(config)).finish();

        let mut builder = ClientBuilder::new();
        if let Some(lang) = lang {
            builder = builder.header(actix_web::http::header::ACCEPT_LANGUAGE, lang);
        }
        if let Some(if_none_match) = if_none_match {
            builder = builder.header(actix_web::http::header::IF_NONE_MATCH, if_none_match);
        }

        builder.connector(connector).finish()
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
            let response = ssl_client(lang, if_none_match).get(url).send().await;
            response
                .map_err(move |err| {
                    error!("HTTP Request failed: {}", err);
                    panic!("Failed to retrieve {}", url2.clone());
                })
                .and_then(|response| {
                    assert_eq!(response.status(), expected);
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

    #[test]
    fn valid_vhost() {
        let _ = env_logger::try_init();

        start_server(7443);

        let _ = request(
            "https://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );

        let _ = lang_request(
            "https://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
            None,
        );

        let _ = lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
            None,
        );

        let _ = lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("fr-FR"),
            None,
        );

        let _ = lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::NOT_FOUND,
            "text/html",
            Some("es-SP"),
            None,
        );

        let _ = lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        );

        let _ = request(
            "https://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        );

        let _ = request(
            "https://valid.localhost:7443/index2.html",
            StatusCode::NOT_FOUND,
            "text/html",
        );

        let _ = request(
            "https://valid.localhost:7443/some/file.txt",
            StatusCode::NOT_FOUND,
            "text/plain",
        );

        let _ = request(
            "https://valid.localhost:7443/manifest.webapp",
            StatusCode::OK,
            "application/json",
        );

        let _ = request(
            "https://valid2.localhost:7443/manifest.webmanifest",
            StatusCode::OK,
            "application/manifest+json",
        );

        let _ = request(
            "https://unknown.localhost:7443/index.html",
            StatusCode::NOT_FOUND,
            "text/html",
        );

        let _ = request(
            "https://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );

        let _ = request(
            "https://missing-zip.localhost:7443/js/main.js",
            StatusCode::OK,
            "application/javascript",
        );

        let _ = lang_request(
            "https://missing-zip.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
            None,
        );

        // Testing etag from file.
        let etag = request(
            "https://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        )
        .unwrap();
        assert_eq!(&etag, "W/\"1600817409.49581208-0\"");

        let etag = request_if_none_match(
            "https://missing-zip.localhost:7443/index.html",
            StatusCode::NOT_MODIFIED,
            "text/html",
            "W/\"1600817409.49581208-0\"",
        )
        .unwrap();
        assert_eq!(&etag, "W/\"1600817409.49581208-0\"");

        // Testing etag from zip.
        let etag = request(
            "https://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        )
        .unwrap();
        assert_eq!(&etag, "W/\"2927261257-87\"");

        let etag = request_if_none_match(
            "https://valid.localhost:7443/css/style.css",
            StatusCode::NOT_MODIFIED,
            "text/css",
            "W/\"2927261257-87\"",
        )
        .unwrap();
        assert_eq!(&etag, "W/\"2927261257-87\"");
    }
}
