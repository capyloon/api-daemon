/// A simple vhost http server.
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use log::info;
use rustls::{
    internal::pemfile::{certs, pkcs8_private_keys},
    NoClientAuth, ServerConfig as RustlsServerConfig,
};
use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;

pub mod config;
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
pub fn start_server(config: &config::Config) {
    let config = config.clone();
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

            let app_data = AppData {
                root_path: config.root_path.clone(),
                csp: config.csp.clone(),
                zips: HashMap::new(),
            };

            HttpServer::new(move || {
                App::new()
                    .data(RwLock::new(app_data.clone()))
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
    fn ssl_client(lang: Option<&str>) -> Client {
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

        builder.connector(connector).finish()
    }

    fn lang_request(url: &str, expected: StatusCode, mime: &str, lang: Option<&'static str>) {
        use actix_web::HttpMessage;
        use log::error;

        let url = url.to_owned();
        let mime = mime.to_owned();
        let fut = async move {
            let url2 = url.clone();
            let response = ssl_client(lang).get(url).send().await;
            response
                .map_err(move |err| {
                    error!("HTTP Request failed: {}", err);
                    panic!("Failed to retrieve {}", url2);
                })
                .and_then(|response| {
                    assert_eq!(response.status(), expected);
                    if response.status() == StatusCode::OK {
                        assert_eq!(response.content_type(), mime);
                    }
                    Ok(())
                })
        };
        let _ = System::new("test").block_on(fut);
    }

    fn request(url: &str, expected: StatusCode, mime: &str) {
        lang_request(url, expected, mime, None)
    }

    #[test]
    fn valid_vhost() {
        let _ = env_logger::try_init();

        start_server(7443);

        request(
            "https://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );

        lang_request(
            "https://valid.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
        );

        lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US"),
        );

        lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("fr-FR"),
        );

        lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::NOT_FOUND,
            "text/html",
            Some("es-SP"),
        );

        lang_request(
            "https://valid.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
        );

        request(
            "https://valid.localhost:7443/css/style.css",
            StatusCode::OK,
            "text/css",
        );

        request(
            "https://valid.localhost:7443/index2.html",
            StatusCode::NOT_FOUND,
            "text/html",
        );

        request(
            "https://valid.localhost:7443/some/file.txt",
            StatusCode::NOT_FOUND,
            "text/plain",
        );

        request(
            "https://valid.localhost:7443/manifest.webapp",
            StatusCode::OK,
            "application/json",
        );

        request(
            "https://valid2.localhost:7443/manifest.webmanifest",
            StatusCode::OK,
            "application/manifest+json",
        );

        request(
            "https://unknown.localhost:7443/index.html",
            StatusCode::NOT_FOUND,
            "text/html",
        );

        request(
            "https://missing-zip.localhost:7443/index.html",
            StatusCode::OK,
            "text/html",
        );

        request(
            "https://missing-zip.localhost:7443/js/main.js",
            StatusCode::OK,
            "application/javascript",
        );

        lang_request(
            "https://missing-zip.localhost:7443/localized.html",
            StatusCode::OK,
            "text/html",
            Some("en-US,en;q=0.5"),
        );
    }
}
