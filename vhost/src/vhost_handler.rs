/// A actix-web vhost handler
use actix_web::http::header::{self, Header};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use log::{debug, error};
use mime_guess::{Mime, MimeGuess};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};
use zip::read::{ZipArchive, ZipFile};

#[derive(Clone)]
pub(crate) struct AppData {
    pub(crate) root_path: String,
    pub(crate) csp: String,
    pub(crate) zips: HashMap<String, Arc<ZipArchive<File>>>,
}

#[inline]
fn internal_server_error() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

// Returns the mime type for a file, with a special case for manifest.webapp so
// it get recognized as a JSON resource.
fn mime_type_for(file_name: &str) -> Mime {
    if file_name != "manifest.webapp" {
        MimeGuess::from_path(file_name).first_or_octet_stream()
    } else {
        // Force an "application/json" mime type for manifest.webapp files
        MimeGuess::from_ext("json").first_or_octet_stream()
    }
}

// Naive conversion of a ZipFile into an HttpResponse.
// TODO: streaming version.
fn response_from_zip<'a>(zip: &mut ZipFile<'a>, csp: &str) -> HttpResponse {
    let mut buf = vec![];
    let _ = zip.read_to_end(&mut buf);

    let mime = mime_type_for(zip.name());

    HttpResponse::Ok()
        .set_header("Content-Security-Policy", csp)
        .content_type(mime.as_ref())
        .body(buf)
}

// Returns a http response for a given file path.
// TODO: streaming version.
fn response_from_file(path: &Path, csp: &str) -> HttpResponse {
    if let Ok(mut file) = File::open(path) {
        let mut buf = vec![];
        if let Err(err) = file.read_to_end(&mut buf) {
            error!("Failed to read {} : {}", path.to_string_lossy(), err);
            return internal_server_error();
        }

        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_string_lossy();
        let mime = mime_type_for(&file_name);
        HttpResponse::Ok()
            .set_header("Content-Security-Policy", csp)
            .content_type(mime.as_ref())
            .body(buf)
    } else {
        HttpResponse::NotFound().finish()
    }
}

// Helper to figure out if we can use a language specific version of
// a given path.
fn check_lang_files<F>(
    languages: &header::AcceptLanguage,
    filename: &str,
    closure: &mut F,
) -> Result<HttpResponse, ()>
where
    F: FnMut(&str) -> Option<HttpResponse>,
{
    let path = Path::new(filename);
    let file_stem = path
        .file_stem()
        .ok_or_else(|| HttpResponse::BadRequest().finish())
        .map_err(|_| {})?
        .to_string_lossy();
    let file_ext = path.extension();

    for lang in languages.iter().map(|e| format!("{}", e.item)) {
        // Build a new filename: main.ext -> main.lang.ext
        let lang_file = if let Some(ext) = file_ext {
            format!("{}.{}.{}", file_stem, lang, ext.to_string_lossy())
        } else {
            format!("{}.{}", file_stem, lang)
        };

        if let Some(response) = closure(&lang_file) {
            return Ok(response);
        }
    }

    Err(())
}

// Maps requests to a static file based on host value:
// http://host:port/path/to/file.ext -> $root_path/host/application.zip!path/to/file.ext
// When using a Gaia debug profile, applications are not packaged so if application.zip doesn't
// exist we try to map to $root_path/host/path/to/file.ext instead.
pub(crate) async fn vhost(
    data: web::Data<RwLock<AppData>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    if let Some(host) = req.headers().get(header::HOST) {
        let host = host
            .to_str()
            .map_err(|_| HttpResponse::BadRequest().finish())?;
        debug!("Full Host is {:?}", host);
        // Remove the port if there is one.
        let mut parts = host.split(':');
        // TODO: make more robust for cases where the host part can include ':' like ipv6 addresses.
        let host = parts
            .next()
            .ok_or_else(|| HttpResponse::BadRequest().finish())?;
        debug!("Host is {:?}", host);

        // Now extract our vhost, which is the leftmost part of the domain name.
        let mut parts = host.split('.');
        let host = parts
            .next()
            .ok_or_else(|| HttpResponse::BadRequest().finish())?;

        let (root_path, csp, has_zip) = {
            let data = data.read().map_err(|_| internal_server_error())?;
            (
                data.root_path.clone(),
                data.csp.clone(),
                data.zips.contains_key(host),
            )
        };

        // Get a sorted list of languages to get the localized version of the resource.
        let mut languages =
            header::AcceptLanguage::parse(&req).map_err(|_| HttpResponse::BadRequest().finish())?;

        // Sort in descending order of quality.
        languages.sort_by(|item1, item2| {
            use std::cmp::Ordering;

            match item1.quality.cmp(&item2.quality) {
                Ordering::Equal => Ordering::Equal,
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
            }
        });

        // Check if we have a ziparchive for this host.
        if !has_zip {
            // We don't track this host yet, check if there is a matching zip.
            let path = std::path::PathBuf::from(format!("{}/{}/application.zip", root_path, host));
            // Check for application.zip existence.
            if let Ok(file) = File::open(path) {
                // Now add it to the map.
                let archive = ZipArchive::new(file).map_err(|_| ())?;
                let mut data = data.write().map_err(|_| internal_server_error())?;
                data.zips.insert(host.into(), Arc::new(archive));
            } else {
                // No application.zip found, try a direct path mapping.
                let filename = req.match_info().query("filename");

                return match check_lang_files(&languages, filename, &mut |lang_file| {
                    let path = format!("{}/{}/{}", root_path, host, lang_file);
                    let full_path = Path::new(&path);
                    if full_path.exists() {
                        debug!("Direct Opening of {}", lang_file);
                        return Some(response_from_file(&full_path, &csp));
                    }
                    None
                }) {
                    Ok(response) => Ok(response),
                    Err(_) => {
                        // Default fallback
                        let path = format!("{}/{}/{}", root_path, host, filename);
                        let full_path = Path::new(&path);
                        if full_path.exists() {
                            debug!("Direct Opening of {}", filename);
                            Ok(response_from_file(&full_path, &csp))
                        } else {
                            Ok(HttpResponse::NotFound().finish())
                        }
                    }
                };
            }
        }

        // Now we are sure the zip archive is in our hashmap.
        // We still need a write lock because ZipFile.by_name()takes a `&mut self` parameter :(
        let mut readlock = data.write().map_err(|_| internal_server_error())?;

        match readlock.zips.get_mut(host) {
            Some(archive) => {
                let archive = Arc::get_mut(&mut *archive).ok_or_else(internal_server_error)?;
                let filename = req.match_info().query("filename");

                match check_lang_files(&languages, filename, &mut |lang_file| {
                    if let Ok(mut zip) = archive.by_name(&lang_file) {
                        debug!("Opening {}", lang_file);
                        return Some(response_from_zip(&mut zip, &csp));
                    }
                    None
                }) {
                    Ok(response) => Ok(response),
                    Err(_) => {
                        // Default fallback
                        match archive.by_name(filename) {
                            Ok(mut zip) => Ok(response_from_zip(&mut zip, &csp)),
                            Err(_) => Ok(HttpResponse::NotFound().finish()),
                        }
                    }
                }
            }
            None => Ok(internal_server_error()),
        }
    } else {
        // No host in the http request -> client error.
        Ok(HttpResponse::BadRequest().finish())
    }
}
