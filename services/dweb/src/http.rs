/// HTTP Handler for broadcasted data.
/// manages urls such as /ticket/xyz
use actix_web::http::header;
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use iroh::get::get_first_blob;
use iroh::provider::Ticket;
use serde::Deserialize;
use std::str::FromStr;
use tokio_util::io::ReaderStream;

pub static TICKET_PATTERN: &str = "/{ticket}";

const MAX_CONCURRENT_DIALS: u8 = 16;

#[derive(Deserialize)]
pub struct Info {
    ticket: String,
}

pub async fn ticket_handler(info: web::Path<Info>) -> impl Responder {
    let ticket = match Ticket::from_str(&info.ticket) {
        Ok(ticket) => ticket,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if let Ok((reader, maybe_mime, content_length)) =
        get_first_blob(&ticket, false, MAX_CONCURRENT_DIALS).await
    {
        let stream = ReaderStream::new(reader);

        let mut builder = HttpResponse::Ok();
        builder.insert_header((header::CONTENT_LENGTH, content_length.to_string()));

        if let Some(mime_type) = maybe_mime {
            // Disable compression by setting ContentEncoding::Identity (see https://docs.rs/actix-web/4.0.0-beta.19/actix_web/middleware/struct.Compress.html)
            // for mime types that represent already compressed data.
            if mime_type != "image/svg+xml"
                && (mime_type.starts_with("image/")
                    || mime_type.starts_with("audio/")
                    || mime_type.starts_with("video/"))
            {
                builder.insert_header(header::ContentEncoding::Identity);
            }

            builder.insert_header((header::CONTENT_TYPE, mime_type));
        }
        builder.streaming(stream)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
