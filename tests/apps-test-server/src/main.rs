/// This simple server is for apps-service client test.
/// The server hosts applications including manifest.webmanifest and zip package
/// Under /apps. Hawk authentication is required and only GET method is supported.
/// For test purpose, client uses fixed mock id and key to generate Hawk header.
/// kid: "FGFYvY+/4XwTYIX9nVi+sXj5tPA=", mac_key: "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk="

use actix_web::{
    middleware, web, App, HttpRequest, HttpResponse,
    HttpServer,
};
use mime_guess::{Mime, MimeGuess};
use std::{env, io};
use hawk::{RequestBuilder, Header, Key, SHA256};
use hawk::mac::Mac;
use std::time::{Duration, UNIX_EPOCH};

use log::{debug, error};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn mime_type_for(file_name: &str) -> Mime {
    MimeGuess::from_path(file_name).first_or_octet_stream()
}

fn response_from_file(name: &str) -> HttpResponse {
    let name_string = format!("apps/{}", name);
    let path = Path::new(&name_string);
    if let Ok(mut file) = File::open(path) {
        // Check if we can return NotModified without reading the file content.
        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_string_lossy();
        let mime = mime_type_for(&file_name);

        let mut buf = vec![];
        if let Err(err) = file.read_to_end(&mut buf) {
            error!("Failed to read {} : {}", path.to_string_lossy(), err);
            return HttpResponse::InternalServerError().finish();
        }

        HttpResponse::Ok()
            .content_type(mime.as_ref())
            .body(buf)
    } else {
        HttpResponse::NotFound().finish()
    }
}

static MAC_KEY: &str = "p7cI80SwX+gmX0G+T938agWAV1eR9wrpCR9JgsoIIlk=";
static PORT: u16 = 8596;
static HOST: &str = "127.0.0.1";

fn validate(req: HttpRequest) -> bool {
    match req
        .headers()
        .get(::actix_web::http::header::AUTHORIZATION)
    {
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
                        let value = item[index+1..item.len()].replace("\"", "");
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
                    Some(UNIX_EPOCH + Duration::new(hawk_auth.get("ts").unwrap().parse::<u64>().unwrap(), 0)),
                    Some(nounce.as_str()),
                    Some(mac),
                    None,
                    None,
                    None,
                    None,
                ).unwrap();

                let request = RequestBuilder::new("GET", HOST, PORT, req.path())
                .request();

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
    web::Path((name,)): web::Path<(String,)>,
) -> HttpResponse {
    if !validate(req) {
        return HttpResponse::Unauthorized().finish();
    }
    response_from_file(&name)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    let addr = "".to_owned() + HOST + ":" + &PORT.to_string();
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/apps/{name}").route(web::get().to(apps_responses)))
            .service(
                web::scope("/")
                    .route("*", web::post().to(HttpResponse::MethodNotAllowed))
            )
    })
    .bind(addr)?
    .run()
    .await
}
