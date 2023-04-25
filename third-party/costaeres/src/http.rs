// HTTP handler allowing GET access to resource variants.
//
// Urls are following the template: http://127.0.0.1:$port/cmgr/$access_key/$resource_id/$variant_name
// The access key is a bound to a service instance to prevent unauthorized access.
// This means that these links should not be bookmarked since they are not generally reusable.

use crate::common::{
    BoxedReader, ResourceId, ResourceKind, ResourceMetadata, ResourceStore, ResourceStoreError,
};
use crate::file_store::FileStore;
use actix_web::http::header;
use actix_web::web::{self, Bytes};
use actix_web::{HttpResponse, Responder};
use async_std::io::ReadExt;
use futures_core::{
    future::Future,
    ready,
    stream::Stream,
    task::{Context, Poll},
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use speedy::Readable;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;

pub static RESOURCE_PATTERN: &str = "/{access_key}/{resource_id}/{variant}";

const CHUNK_SIZE: usize = 64 * 1024;

struct ChunkedReader {
    reader: BoxedReader,
}

impl ChunkedReader {
    fn new(reader: BoxedReader) -> Self {
        Self { reader }
    }
}

impl Stream for ChunkedReader {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

        let read = ready!(Future::poll(
            Pin::new(&mut self.reader.read(&mut buffer)),
            cx
        ))?;

        if read == 0 {
            // We reached EOF
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Self::Item::Ok(Bytes::copy_from_slice(
                &buffer[0..read],
            ))))
        }
    }
}

pub struct HttpData {
    pub store: FileStore,
    pub keys: Arc<Mutex<HashSet<String>>>,
}

#[derive(Deserialize)]
pub struct Info {
    access_key: String,
    resource_id: String,
    variant: String,
}

#[derive(Serialize, Deserialize)]
struct MetaSummary {
    id: String,
    name: String,
    container: bool,
    tags: Vec<String>,
}

impl From<ResourceMetadata> for MetaSummary {
    fn from(meta: ResourceMetadata) -> Self {
        Self {
            id: meta.id().into(),
            name: meta.name(),
            container: meta.kind() == ResourceKind::Container,
            tags: meta.tags().clone(),
        }
    }
}

pub async fn resource_handler(data: web::Data<HttpData>, info: web::Path<Info>) -> impl Responder {
    // Check the key, and fail with error 400 if it's invalid.
    {
        let keys = data.keys.lock();
        if !keys.contains(&info.access_key) {
            return HttpResponse::BadRequest().finish();
        }
    }

    let store = &data.store;

    match store
        .get_full(&ResourceId::from(info.resource_id.clone()), &info.variant)
        .await
    {
        Err(ResourceStoreError::NoSuchResource) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
        Ok((meta, mut reader)) => {
            let mut response = HttpResponse::Ok();
            // Find mime type and size from the metadata.
            for variant in meta.variants() {
                if variant.name() == info.variant {
                    let mime_type = variant.mime_type();

                    // Disable compression by setting ContentEncoding::Identity (see https://docs.rs/actix-web/4.0.0-beta.19/actix_web/middleware/struct.Compress.html)
                    // for mime types that represent already compressed data.
                    if mime_type != "image/svg+xml"
                        && (mime_type.starts_with("image/")
                            || mime_type.starts_with("audio/")
                            || mime_type.starts_with("video/"))
                    {
                        response.insert_header(header::ContentEncoding::Identity);
                    }

                    response.insert_header((header::CONTENT_TYPE, mime_type));
                    response.insert_header((header::CONTENT_LENGTH, variant.size().to_string()));

                    break;
                }
            }

            if meta.kind() == ResourceKind::Container {
                response.insert_header((header::CONTENT_TYPE, "application/json"));

                let mut buffer = vec![];
                reader.read_to_end(&mut buffer).await.unwrap_or(0);
                let children = Vec::<ResourceId>::read_from_buffer(&buffer).unwrap_or_default();

                let mut list = vec![];
                for child in children {
                    if let Ok(meta) = store.get_metadata(&child).await {
                        list.push(MetaSummary::from(meta));
                    }
                }

                response.body(serde_json::to_string(&list).unwrap_or_else(|_| "[]".into()))
            } else {
                response.streaming(ChunkedReader::new(reader))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::common::*;
    use crate::http::*;
    use actix_web::http::header;
    use actix_web::http::StatusCode;
    use actix_web::web::{Bytes, Data};
    use actix_web::{test, App};
    use async_std::fs;
    use speedy::Writable;

    fn named_variant(name: &str) -> VariantMetadata {
        VariantMetadata::new(name, "application/octet-stream", 42)
    }

    fn default_variant() -> VariantMetadata {
        named_variant("default")
    }

    async fn named_content(name: &str) -> Variant {
        let file = fs::File::open("./create_db.sh").await.unwrap();
        Variant::new(named_variant(name), Box::new(file))
    }

    async fn default_content() -> Variant {
        named_content("default").await
    }

    async fn add_root(store: &FileStore) {
        // Adding an object.
        let meta = ResourceMetadata::new(
            &ROOT_ID,
            &ROOT_ID,
            ResourceKind::Leaf,
            "object 0",
            vec!["one".into(), "two".into()],
            vec![default_variant()],
        );

        store
            .create(&meta, Some(default_content().await))
            .await
            .unwrap();
    }

    async fn add_root_container(store: &FileStore) {
        // Adding the root object.
        let meta = ResourceMetadata::new(
            &ROOT_ID,
            &ROOT_ID,
            ResourceKind::Container,
            "/",
            vec![],
            vec![VariantMetadata::new("default", "inode/directory", 0)],
        );

        store.create(&meta, None).await.unwrap();

        let mut ids = vec![];
        for id in 0..10 {
            let meta = ResourceMetadata::new(
                &(format!("child-{id}").into()),
                &ROOT_ID,
                ResourceKind::Leaf,
                &format!("child-{id}"),
                vec!["one".into(), "two".into()],
                vec![default_variant()],
            );

            store
                .create(&meta, Some(default_content().await))
                .await
                .unwrap();

            ids.push(meta.id());
        }

        // Update the root container content.
        store
            .update_default_variant_from_slice(&ROOT_ID, &ids.write_to_vec().unwrap())
            .await
            .unwrap();
    }

    async fn get_data(path: &str) -> HttpData {
        let _ = fs::remove_dir_all(path).await;
        let _ = fs::create_dir_all(path).await;

        HttpData {
            store: FileStore::new(
                path,
                Box::new(DefaultResourceNameProvider),
                Box::new(IdentityTransformer),
            )
            .await
            .unwrap(),
            keys: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    macro_rules! create_app {
        ($data:expr) => {
            test::init_service(
                App::new().service(
                    web::scope("/cmgr")
                        .app_data(Data::new($data))
                        .route("*", web::post().to(HttpResponse::MethodNotAllowed))
                        .route(RESOURCE_PATTERN, web::get().to(resource_handler)),
                ),
            )
            .await
        };
    }

    #[actix_rt::test]
    async fn http_wrong_path() {
        let data = get_data("./http-test-content/0").await;

        let app = create_app!(data);

        let req = test::TestRequest::get().uri("/random/path").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn http_wrong_key() {
        let data = get_data("./http-test-content/1").await;

        {
            let mut keys = data.keys.lock();
            keys.insert("somekey".into());
        }

        let app = create_app!(data);

        let req = test::TestRequest::get()
            .uri("/cmgr/key1/resource1/default")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn http_correct_key() {
        let data = get_data("./http-test-content/2").await;

        {
            {
                let mut keys = data.keys.lock();
                keys.insert("somekey".into());
            }

            add_root(&data.store).await;
        }

        let app = create_app!(data);

        let req = test::TestRequest::get()
            .uri(&format!("/cmgr/somekey/{ROOT_ID_STR}/default"))
            .to_request();

        let result = test::call_and_read_body(&app, req).await;
        assert_eq!(result, Bytes::from_static(b"#!/bin/bash\n\nset -x -e\n\nrm build.sqlite\nsqlite3 build.sqlite < db/migrations/00001_main.sql\n\n"));
    }

    #[actix_rt::test]
    async fn http_wrong_resource() {
        let data = get_data("./http-test-content/3").await;

        {
            {
                let mut keys = data.keys.lock();
                keys.insert("somekey".into());
            }

            add_root(&data.store).await;
        }

        let app = create_app!(data);

        let req = test::TestRequest::get()
            .uri("/cmgr/somekey/deadbeef/default")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn http_wrong_variant() {
        let data = get_data("./http-test-content/4").await;

        {
            {
                let mut keys = data.keys.lock();
                keys.insert("somekey".into());
            }

            add_root(&data.store).await;
        }

        let app = create_app!(data);

        let req = test::TestRequest::get()
            .uri(&format!("/cmgr/somekey/{ROOT_ID_STR}/some-variant"))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn http_mime_and_length() {
        let data = get_data("./http-test-content/5").await;

        {
            {
                let mut keys = data.keys.lock();
                keys.insert("somekey".into());
            }

            add_root(&data.store).await;
        }

        let app = create_app!(data);

        let req = test::TestRequest::get()
            .uri(&format!("/cmgr/somekey/{ROOT_ID_STR}/default"))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let headers = resp.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/octet-stream"
        );
        assert_eq!(headers.get(header::CONTENT_LENGTH).unwrap(), "42");
    }

    #[actix_rt::test]
    async fn directory_content() {
        let data = get_data("./http-test-content/6").await;

        {
            {
                let mut keys = data.keys.lock();
                keys.insert("somekey".into());
            }

            add_root_container(&data.store).await;
        }

        let app = create_app!(data);

        let req = test::TestRequest::get()
            .uri(&format!("/cmgr/somekey/{ROOT_ID_STR}/default"))
            .to_request();

        // Check that it's a json mime type.
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let headers = resp.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let req = test::TestRequest::get()
            .uri(&format!("/cmgr/somekey/{ROOT_ID_STR}/default"))
            .to_request();

        // Check json content.
        let result = test::call_and_read_body(&app, req).await;

        let metas: Vec<MetaSummary> = serde_json::from_slice(&result).unwrap();

        assert_eq!(metas.len(), 10);
    }
}
