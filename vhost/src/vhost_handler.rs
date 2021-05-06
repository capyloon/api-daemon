/// A actix-web vhost handler
use crate::etag::*;
use actix_web::http::header::{self, Header, HeaderValue};
use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use common::traits::Shared;
use log::{debug, error};
use mime_guess::{Mime, MimeGuess};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::read::{ZipArchive, ZipFile};
use zip::CompressionMethod;

#[derive(Default)]
pub struct AppData {
    pub root_path: String,
    pub csp: String,
    // Map a host name to the zip.
    pub zips: HashMap<String, ZipArchive<File>>,
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

pub fn maybe_not_modified(
    if_none_match: Option<&HeaderValue>,
    etag: &str,
    mime: &Mime,
    csp: Option<&str>,
) -> Option<HttpResponse> {
    // Check if we have an etag from the If-None-Match header.
    if let Some(if_none_match) = if_none_match {
        if let Ok(value) = if_none_match.to_str() {
            if etag == value {
                let mut resp304 = HttpResponse::NotModified();
                let mut builder = resp304
                    .content_type(mime.as_ref())
                    .set_header("ETag", etag.to_string());
                if let Some(csp) = csp {
                    builder = builder.set_header("Content-Security-Policy", csp);
                }
                return Some(builder.finish());
            }
        }
    }
    None
}

// Naive conversion of a ZipFile into an HttpResponse.
// TODO: streaming version.
fn response_from_zip<'a>(
    zip: &mut ZipFile<'a>,
    csp: &str,
    if_none_match: Option<&HeaderValue>,
) -> HttpResponse {
    // Check if we can return NotModified without reading the file content.
    let etag = Etag::for_zip(&zip);
    let mime = mime_type_for(zip.name());
    if let Some(response) = maybe_not_modified(if_none_match, &etag, &mime, Some(csp)) {
        return response;
    }

    let mut buf = vec![];
    let _ = zip.read_to_end(&mut buf);

    let mut ok = HttpResponse::Ok();
    let builder = ok
        .set_header("Content-Security-Policy", csp)
        .set_header("ETag", etag)
        .content_type(mime.as_ref());

    // If the zip entry was compressed, set the content type accordingly.
    // if not, the actix middleware will do its own compression based
    // on the supported content-encoding supplied by the client.
    if zip.compression() == CompressionMethod::Deflated {
        builder.set_header("Content-Encoding", "deflate");
    }
    builder.body(buf)
}

// Returns a http response for a given file path.
// TODO: streaming version.
fn response_from_file(path: &Path, csp: &str, if_none_match: Option<&HeaderValue>) -> HttpResponse {
    if let Ok(mut file) = File::open(path) {
        // Check if we can return NotModified without reading the file content.
        let etag = Etag::for_file(&file);
        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_string_lossy();
        let mime = mime_type_for(&file_name);
        if let Some(response) = maybe_not_modified(if_none_match, &etag, &mime, Some(csp)) {
            return response;
        }

        let mut buf = vec![];
        if let Err(err) = file.read_to_end(&mut buf) {
            error!("Failed to read {} : {}", path.to_string_lossy(), err);
            return internal_server_error();
        }

        HttpResponse::Ok()
            .set_header("Content-Security-Policy", csp)
            .content_type(mime.as_ref())
            .set_header("ETag", etag)
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

    // file_stem only contains the file name, so we need to prepend the path
    // if any.
    let parent = match path.parent() {
        Some(parent) => {
            let disp = format!("{}", parent.display());
            if !disp.is_empty() {
                format!("{}/", disp)
            } else {
                disp
            }
        }
        None => "".to_owned(),
    };

    for lang in languages.iter().map(|e| format!("{}", e.item)) {
        // Build a new filename: main.ext -> main.lang.ext
        let lang_file = if let Some(ext) = file_ext {
            format!("{}{}.{}.{}", parent, file_stem, lang, ext.to_string_lossy())
        } else {
            format!("{}{}.{}", parent, file_stem, lang)
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
pub async fn vhost(
    data: web::Data<Shared<AppData>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let if_none_match = req.headers().get(header::IF_NONE_MATCH);

    if let Some(host) = req.headers().get(header::HOST) {
        let full_host = host
            .to_str()
            .map_err(|_| HttpResponse::BadRequest().finish())?;
        debug!("Full Host is {:?}", full_host);
        // Remove the port if there is one.
        let mut parts = full_host.split(':');
        // TODO: make more robust for cases where the host part can include ':' like ipv6 addresses.
        let host = parts
            .next()
            .ok_or_else(|| HttpResponse::BadRequest().finish())?;
        debug!("Host is {:?}", host);

        if host == "localhost" {
            // mapping the redirect url
            // from: http://localhost[:port]/redirect/app/file.html?...
            // to: http://app.localhost[:port]/file.html?...
            let path = req.path();
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() < 2 || parts[1] != "redirect" {
                return Ok(HttpResponse::BadRequest().finish());
            }
            let replace_path = format!("{}/{}/", parts[1], parts[2]);
            let path = path.replace(&replace_path, "");
            let params = req.query_string();
            let redirect_url = format!("http://{}.{}{}?{}", parts[2], full_host, path, params);
            return Ok(HttpResponse::MovedPermanently()
                .header(http::header::LOCATION, redirect_url)
                .finish());
        }

        // Now extract our vhost, which is the leftmost part of the domain name.
        let mut parts = host.split('.');
        let host = parts
            .next()
            .ok_or_else(|| HttpResponse::BadRequest().finish())?;

        let (root_path, csp, has_zip) = {
            let data = data.lock();
            (
                data.root_path.clone(),
                data.csp.clone(),
                data.zips.contains_key(host),
            )
        };

        // Replace instances of 'self' in the CSP by the current origin.
        // This is needed for proper loading of aliased URIs like about:neterror
        let csp_self = format!("http://{}", full_host);
        let csp = csp.replace("'self'", &csp_self);

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

        // Check if we have a zip archive for this host.
        if !has_zip {
            // We don't track this host yet, check if there is a matching zip.
            let path = std::path::PathBuf::from(format!("{}/{}/application.zip", root_path, host));
            // Check for application.zip existence.
            if let Ok(file) = File::open(path) {
                // Now add it to the map.
                let archive = ZipArchive::new(file).map_err(|_| ())?;
                let mut data = data.lock();
                data.zips.insert(host.into(), archive);
            } else {
                // No application.zip found, try a direct path mapping.
                let filename = req.match_info().query("filename");

                return match check_lang_files(&languages, filename, &mut |lang_file| {
                    let path = format!("{}/{}/{}", root_path, host, lang_file);
                    let full_path = Path::new(&path);
                    if full_path.exists() {
                        debug!("Direct Opening of {}", lang_file);
                        return Some(response_from_file(&full_path, &csp, if_none_match));
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
                            Ok(response_from_file(&full_path, &csp, if_none_match))
                        } else {
                            Ok(HttpResponse::NotFound().finish())
                        }
                    }
                };
            }
        }

        // Now we are sure the zip archive is in our hashmap.
        // We still need a write lock because ZipFile.by_name_maybe_raw()takes a `&mut self` parameter :(
        let mut readlock = data.lock();

        match readlock.zips.get_mut(host) {
            Some(archive) => {
                let filename = req.match_info().query("filename");

                match check_lang_files(&languages, filename, &mut |lang_file| {
                    if let Ok(mut zip) =
                        archive.by_name_maybe_raw(&lang_file, CompressionMethod::Deflated)
                    {
                        debug!("Opening {}", lang_file);
                        return Some(response_from_zip(&mut zip, &csp, if_none_match));
                    }
                    None
                }) {
                    Ok(response) => Ok(response),
                    Err(_) => {
                        // Default fallback
                        match archive.by_name_maybe_raw(filename, CompressionMethod::Deflated) {
                            Ok(mut zip) => Ok(response_from_zip(&mut zip, &csp, if_none_match)),
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
