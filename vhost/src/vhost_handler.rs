/// A actix-web vhost handler
use crate::etag::*;
use actix_web::http::header::{self, Header, HeaderValue};
use actix_web::{http, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use async_trait::async_trait;
use common::traits::Shared;
use log::debug;
use mime_guess::{Mime, MimeGuess};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::fs::File as AsyncFile;
use tokio_util::io::ReaderStream;
use zip::read::{ZipArchive, ZipFile};
use zip::CompressionMethod;

// Chunk size when streaming files.
const CHUNK_SIZE: usize = 16 * 1024;

#[derive(Default)]
pub struct AppData {
    pub root_path: String,
    pub csp: String,
    // Map a host name to the zip.
    pub zips: HashMap<String, ZipArchive<File>>,
    // from -> to subdomain mapping
    pub mappings: HashMap<String, String>,
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
                    .insert_header(("ETag", etag.to_string()));
                if let Some(csp) = csp {
                    builder = builder.insert_header(("Content-Security-Policy", csp));
                }
                return Some(builder.finish());
            }
        }
    }
    None
}

// Resources that are already compressed formats don't benefit from being compressed again,
// so force the use of the 'Identity' content encoding for these.
fn update_response_encoding(mime: &Mime, builder: &mut HttpResponseBuilder) {
    let mime_str = mime.as_ref();
    if mime_str != "image/svg+xml"
        && (mime_str.starts_with("image/")
            || mime_str.starts_with("audio/")
            || mime_str.starts_with("video/"))
    {
        builder.insert_header(header::ContentEncoding::Identity);
    }
}

// Naive conversion of a ZipFile into an HttpResponse.
// TODO: streaming version.
fn response_from_zip<'a>(
    zip: &mut ZipFile<'a>,
    csp: &str,
    if_none_match: Option<&HeaderValue>,
) -> HttpResponse {
    // Check if we can return NotModified without reading the file content.
    let etag = Etag::for_zip(zip);
    let mime = mime_type_for(zip.name());
    if let Some(response) = maybe_not_modified(if_none_match, &etag, &mime, Some(csp)) {
        return response;
    }

    let mut buf = vec![];
    let _ = zip.read_to_end(&mut buf);

    let mut ok = HttpResponse::Ok();
    let builder = ok
        .insert_header(("Content-Security-Policy", csp))
        .insert_header(("ETag", etag))
        .content_type(mime.as_ref());

    // If the zip entry was compressed, set the content type accordingly.
    // if not, the actix middleware will do its own compression based
    // on the supported content-encoding supplied by the client.
    if zip.compression() == CompressionMethod::Deflated {
        builder.insert_header(("Content-Encoding", "deflate"));
    } else {
        update_response_encoding(&mime, builder);
    }
    builder.body(buf)
}

// Returns a http response for a given file path.
async fn response_from_file(
    path: &Path,
    csp: &str,
    if_none_match: Option<&HeaderValue>,
) -> HttpResponse {
    if let Ok(mut file) = AsyncFile::open(path).await {
        // Check if we can return NotModified without reading the file content.
        let etag = Etag::for_file(&mut file).await;
        let file_name = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(""))
            .to_string_lossy();
        let mime = mime_type_for(&file_name);
        if let Some(response) = maybe_not_modified(if_none_match, &etag, &mime, Some(csp)) {
            return response;
        }

        let mut builder = HttpResponse::Ok();
        builder
            .insert_header(("Content-Security-Policy", csp))
            .content_type(mime.as_ref())
            .insert_header(("ETag", etag));

        update_response_encoding(&mime, &mut builder);

        builder.streaming(ReaderStream::with_capacity(file, CHUNK_SIZE))
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[async_trait(?Send)]
trait LangChecker {
    async fn check(&mut self, lang_file: &str) -> Option<HttpResponse>;
}

// Helper to figure out if we can use a language specific version of
// a given path.
async fn check_lang_files<C: LangChecker>(
    languages: &header::AcceptLanguage,
    filename: &str,
    mut checker: C,
) -> Result<HttpResponse, ()> {
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

        if let Some(response) = checker.check(&lang_file).await {
            return Ok(response);
        }
    }

    Err(())
}

struct FileLangChecker<'a> {
    root_path: String,
    host: String,
    csp: String,
    if_none_match: Option<&'a HeaderValue>,
}

#[async_trait(?Send)]
impl<'a> LangChecker for FileLangChecker<'a> {
    async fn check(&mut self, lang_file: &str) -> Option<HttpResponse> {
        let path = format!("{}/{}/{}", self.root_path, self.host, lang_file);
        let full_path = Path::new(&path);
        if full_path.exists() {
            debug!("Direct Opening of {}", lang_file);
            return Some(response_from_file(full_path, &self.csp, self.if_none_match).await);
        }
        None
    }
}

use std::sync::Arc;
struct ZipLangChecker<'a> {
    archive: Arc<&'a mut ZipArchive<std::fs::File>>,
    csp: String,
    if_none_match: Option<&'a HeaderValue>,
}

#[async_trait(?Send)]
impl<'a> LangChecker for ZipLangChecker<'a> {
    async fn check(&mut self, lang_file: &str) -> Option<HttpResponse> {
        if let Some(archive) = Arc::get_mut(&mut self.archive) {
            if let Ok(mut zip) = archive.by_name_maybe_raw(lang_file, CompressionMethod::Deflated) {
                debug!("Opening {}", lang_file);
                return Some(response_from_zip(&mut zip, &self.csp, self.if_none_match));
            }
        }
        None
    }
}

// Maps requests to a static file based on host value:
// http://host:port/path/to/file.ext -> $root_path/host/application.zip!path/to/file.ext
// When using a Gaia debug profile, applications are not packaged so if application.zip doesn't
// exist we try to map to $root_path/host/path/to/file.ext instead.
pub async fn vhost(data: web::Data<Shared<AppData>>, req: HttpRequest) -> impl Responder {
    let if_none_match = req.headers().get(header::IF_NONE_MATCH);

    if let Some(host) = req.headers().get(header::HOST) {
        let full_host = match host.to_str() {
            Ok(full_host) => full_host,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };
        debug!("Full Host is {:?}", full_host);
        // Remove the port if there is one.
        let mut parts = full_host.split(':');
        // TODO: make more robust for cases where the host part can include ':' like ipv6 addresses.
        let host = match parts.next() {
            Some(host) => host,
            None => return HttpResponse::BadRequest().finish(),
        };
        let maybe_port = parts.next();
        debug!("Host is {:?} port={:?}", host, maybe_port);

        if host == "localhost" {
            // mapping the redirect url
            // from: http://localhost[:port]/redirect/app/file.html?...
            // to: http://app.localhost[:port]/file.html?...
            let path = req.path();
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() < 2 || parts[1] != "redirect" {
                return HttpResponse::BadRequest().finish();
            }
            let replace_path = format!("{}/{}/", parts[1], parts[2]);
            let path = path.replace(&replace_path, "");
            let params = req.query_string();
            let redirect_url = format!("http://{}.{}{}?{}", parts[2], full_host, path, params);
            return HttpResponse::MovedPermanently()
                .insert_header((http::header::LOCATION, redirect_url))
                .finish();
        }

        // Now extract our vhost, which is the leftmost part of the domain name.
        let mut parts = host.split('.');
        let host = match parts.next() {
            Some(host) => host,
            None => return HttpResponse::BadRequest().finish(),
        };

        let (root_path, csp, has_zip, mapped_host) = {
            let data = data.lock();
            (
                data.root_path.clone(),
                data.csp.clone(),
                data.zips.contains_key(host),
                data.mappings.get(host).map(|s| s.to_owned()),
            )
        };

        // Update host and full_host to take mapping into account.
        let (host, full_host) = if let Some(mapped_host) = mapped_host {
            let mapped_full_host = match maybe_port {
                Some(port) => format!("{}.localhost:{}", mapped_host, port),
                None => mapped_host.clone(),
            };
            (mapped_host, mapped_full_host)
        } else {
            (host.to_owned(), full_host.to_owned())
        };

        // Replace instances of 'self' in the CSP by the current origin.
        // This is needed for proper loading of aliased URIs like about:neterror
        let csp_self = format!("http://{}", full_host);
        let csp = csp.replace("'self'", &csp_self);

        // Get a sorted list of languages to get the localized version of the resource.
        let mut languages = match header::AcceptLanguage::parse(&req) {
            Ok(languages) => languages,
            Err(_) => return HttpResponse::BadRequest().finish(),
        };

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
                let archive = match ZipArchive::new(file) {
                    Ok(archive) => archive,
                    Err(_) => return HttpResponse::BadRequest().finish(),
                };
                let mut data = data.lock();
                data.zips.insert(host.clone(), archive);
            } else {
                // No application.zip found, try a direct path mapping.
                // Use the full url except the leading / as the filename, to keep the url parameters if any.
                let filename = &req.uri().to_string()[1..];

                let lang_checker = FileLangChecker {
                    root_path: root_path.clone(),
                    host: host.clone(),
                    csp: csp.clone(),
                    if_none_match: if_none_match.clone(),
                };

                return match check_lang_files(&languages, filename, lang_checker).await {
                    Ok(response) => response,
                    Err(_) => {
                        // Default fallback
                        let path = format!("{}/{}/{}", root_path, host, filename);
                        let full_path = Path::new(&path);
                        if full_path.exists() {
                            debug!("Direct Opening of {}", filename);
                            response_from_file(full_path, &csp, if_none_match).await
                        } else {
                            HttpResponse::NotFound().finish()
                        }
                    }
                };
            }
        }

        // Now we are sure the zip archive is in our hashmap.
        // We still need a write lock because ZipFile.by_name_maybe_raw()takes a `&mut self` parameter :(
        let mut readlock = data.lock();

        match readlock.zips.get_mut(&host) {
            Some(archive) => {
                let filename = req.match_info().query("filename");

                let lang_checker = ZipLangChecker {
                    archive: Arc::new(archive),
                    csp: csp.clone(),
                    if_none_match: if_none_match.clone(),
                };
                match check_lang_files(&languages, filename, lang_checker).await {
                    Ok(response) => response,
                    Err(_) => {
                        // Default fallback
                        match archive.by_name_maybe_raw(filename, CompressionMethod::Deflated) {
                            Ok(mut zip) => response_from_zip(&mut zip, &csp, if_none_match),
                            Err(_) => HttpResponse::NotFound().finish(),
                        }
                    }
                }
            }
            None => internal_server_error(),
        }
    } else {
        // No host in the http request -> client error.
        HttpResponse::BadRequest().finish()
    }
}
