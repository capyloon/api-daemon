use actix_web::http::header::{self, HeaderValue};
/// This simple server is for apps-service client test.
/// The server hosts applications including manifest.webmanifest and zip package
/// Under /apps. Hawk authentication is required and only GET method is supported.
/// For test purpose, client uses fixed mock id and key to generate Hawk header.
/// kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk="
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use hawk::mac::Mac;
use hawk::{Header, Key, RequestBuilder, SHA256};
use mime_guess::{Mime, MimeGuess};
use std::time::{Duration, UNIX_EPOCH};
use std::{env, io};
use vhost_server::etag::*;

use log::{debug, error};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn mime_type_for(file_name: &str) -> Mime {
    MimeGuess::from_path(file_name).first_or_octet_stream()
}

fn maybe_not_modified(
    if_none_match: Option<&HeaderValue>,
    etag: &str,
    mime: &Mime,
) -> Option<HttpResponse> {
    // Check if we have an etag from the If-None-Match header.
    if let Some(if_none_match) = if_none_match {
        if let Ok(value) = if_none_match.to_str() {
            if etag == value {
                let mut resp304 = HttpResponse::NotModified();
                let builder = resp304
                    .content_type(mime.as_ref())
                    .set_header("ETag", etag.to_string());

                return Some(builder.finish());
            }
        }
    }
    None
}

fn response_from_file(req: HttpRequest, app: &str, name: &str) -> HttpResponse {
    let name_string = format!("apps/{}/{}", app, name);
    let path = Path::new(&name_string);
    if let Ok(mut file) = File::open(path) {
        // Check if we can return NotModified without reading the file content.
        let if_none_match = req.headers().get(header::IF_NONE_MATCH);
        let etag = Etag::for_file(&file);
        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_string_lossy();
        let mime = mime_type_for(&file_name);
        if let Some(response) = maybe_not_modified(if_none_match, &etag, &mime) {
            return response;
        }

        let mut buf = vec![];
        if let Err(err) = file.read_to_end(&mut buf) {
            error!("Failed to read {} : {}", path.to_string_lossy(), err);
            return HttpResponse::InternalServerError().finish();
        }

        HttpResponse::Ok()
            .set_header("ETag", etag.to_string())
            .content_type(mime.as_ref())
            .body(buf)
    } else {
        HttpResponse::NotFound().finish()
    }
}

static MAC_KEY: &str = "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=";
static PORT: u16 = 8596;
static HOST: &str = "127.0.0.1";
// This UA is defined in daemon/config.toml.
static EXPECTED_UA: &str = "Mozilla/5.0 (Mobile; rv:90.0) Gecko/90.0 Firefox/90.0 KAIOS/3.2";
static EXPECTED_LANG: &str = "en-US";

fn check_header(req: &HttpRequest, header: header::HeaderName, expected: &str) -> bool {
    match req.headers().get(header) {
        Some(value) => match value.to_str() {
            Ok(value) => value == expected,
            Err(_) => false,
        },
        None => false,
    }
}

fn validate(req: &HttpRequest) -> bool {
    match req.headers().get(header::AUTHORIZATION) {
        Some(header_value) => match header_value.to_str() {
            Ok(value) => {
                let values: Vec<_> = value.split(',').map(|e| e.trim()).collect();
                debug!("AUTHORIZATION is {:?}", values.clone());
                let mut hawk_auth: HashMap<String, String> = HashMap::new();
                // token_type: "hawk", scope: "u|core:cruds sc#apps:rs sc#metrics:c payment#products:rs payment#purchases:crud simcustm#pack:s simcustm#packfile:r payment#transactions:cr payment#prices:s payment#options:s", expires_in: 604800, kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=", mac_algorithm: "hmac-sha-256" }
                //["Hawk id=\"FGFYvY+/4XwTYIX9nVi+sXj5tPA=\"", "ts=\"1611717940\"", "nonce=\"SrnmiS6u9dckTg==\"", "mac=\"gVH14LHIxSTD/Oq7+MsFCpxHzafWRDSEvXlGFnpQAzM=\"", "hash=\"\""]
                for item in values.iter() {
                    if let Some(index) = item.find('=') {
                        let key = item[0..index].replace(" ", "");
                        let value = item[index + 1..item.len()].replace("\"", "");
                        hawk_auth.insert(key, value);
                    }
                }
                debug!("hawk_auth is {:?}", hawk_auth);
                let id = hawk_auth.get("Hawkid").unwrap();
                let mac_string = hawk_auth.get("mac").unwrap();
                let mac = Mac::from(base64::decode(&mac_string).unwrap());
                let nounce = hawk_auth.get("nonce").unwrap();
                let hdr = Header::new(
                    Some(id.as_str()),
                    Some(
                        UNIX_EPOCH
                            + Duration::new(
                                hawk_auth.get("ts").unwrap().parse::<u64>().unwrap(),
                                0,
                            ),
                    ),
                    Some(nounce.as_str()),
                    Some(mac),
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();

                let request = RequestBuilder::new("GET", HOST, PORT, req.path()).request();

                let key = Key::new(base64::decode(MAC_KEY).unwrap(), SHA256).unwrap();
                let one_week_in_secs = 7 * 24 * 60 * 60;

                request.validate_header(&hdr, &key, Duration::from_secs(one_week_in_secs))
            }
            Err(_) => false,
        },
        None => false,
    }
}

async fn apps_responses(
    req: HttpRequest,
    web::Path((app, name)): web::Path<(String, String)>,
) -> HttpResponse {
    // For cancel API test
    std::thread::sleep(std::time::Duration::from_millis(200));
    if !check_header(&req, header::USER_AGENT, EXPECTED_UA) {
        return HttpResponse::BadRequest().finish();
    }
    if !check_header(&req, header::ACCEPT_LANGUAGE, EXPECTED_LANG) {
        return HttpResponse::BadRequest().finish();
    }
    // Do not check the authorization header for pwa.
    if app != "pwa" && !validate(&req) {
        return HttpResponse::Unauthorized().finish();
    }
    response_from_file(req, &app, &name)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    let addr = "".to_owned() + HOST + ":" + &PORT.to_string();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/apps/{app}/{name:[^{}]+}").route(web::get().to(apps_responses)),
            )
            .service(web::scope("/").route("*", web::post().to(HttpResponse::MethodNotAllowed)))
    })
    .bind(addr)?
    .run()
    .await
}
