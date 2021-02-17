/// Indexers for recognized mime types.
use crate::common::{ResourceMetadata, TransactionResult, VariantContent};
use crate::fts::Fts;
use async_std::io::{ReadExt, SeekFrom};
use async_trait::async_trait;
use futures::AsyncSeekExt;
use serde_json::Value;
use sqlx::{Sqlite, Transaction};

#[async_trait(?Send)]
pub trait Indexer {
    async fn index<'c>(
        &self,
        meta: &ResourceMetadata,
        variant: &mut VariantContent,
        fts: &Fts,
        mut tx: Transaction<'c, Sqlite>,
    ) -> TransactionResult<'c>;
}

// A generic indexer for flat Json data structures.
// Indexed properties are strings and string arrays members.
pub struct FlatJsonIndexer {
    fields: Vec<String>,
    mime_type: String,
}

impl FlatJsonIndexer {
    pub fn new(mime_type: &str, fields: &[&str]) -> Self {
        Self {
            fields: fields.iter().cloned().map(|e| e.to_owned()).collect(),
            mime_type: mime_type.into(),
        }
    }
}

#[async_trait(?Send)]
impl Indexer for FlatJsonIndexer {
    async fn index<'c>(
        &self,
        meta: &ResourceMetadata,
        variant: &mut VariantContent,
        fts: &Fts,
        mut tx: Transaction<'c, Sqlite>,
    ) -> TransactionResult<'c> {
        // 0. Filter by mime type.
        if self.mime_type != variant.0.mime_type() {
            return Ok(tx);
        }

        let content = &mut variant.1;

        // 1. Read the content as json.
        content.seek(SeekFrom::Start(0)).await?;
        let mut buffer = vec![];
        content.read_to_end(&mut buffer).await?;
        let v: Value = serde_json::from_slice(&buffer)?;

        // 2. Index each available field.
        for field in &self.fields {
            match v.get(field) {
                Some(Value::String(text)) => {
                    tx = fts
                        .add_text(&meta.id(), &variant.0.name(), text, tx)
                        .await?;
                }
                Some(Value::Array(array)) => {
                    for item in array {
                        if let Value::String(text) = item {
                            tx = fts
                                .add_text(&meta.id(), &variant.0.name(), text, tx)
                                .await?;
                        }
                    }
                }
                _ => {}
            }
        }
        // 3. Re-position the stream at the beginning.
        content.seek(SeekFrom::Start(0)).await?;

        Ok(tx)
    }
}

// Indexer for the content of a "Places" object.
// This is a json value with the following format:
// { url: "...", title: "...", icon: "..." }
pub fn create_places_indexer() -> FlatJsonIndexer {
    FlatJsonIndexer::new("application/x-places+json", &["url", "title"])
}

// Indexer for the content of a "Contacts" object.
// This is a json value with the following format:
// { name: "...", phone: "[...]", email: "[...]" }
pub fn create_contacts_indexer() -> FlatJsonIndexer {
    FlatJsonIndexer::new("application/x-contact+json", &["name", "phone", "email"])
}

// Indexer for the content of a "Media" object.
// This is a json value with the following format:
// {"url":"https://beatbump.ml/search/echoes%2520pink%2520floyd?filter=EgWKAQIIAWoKEAMQBBAKEAkQBQ%3D%3D",
//  "icon":"https://beatbump.ml/logo-header.png",
//  "title":"Echoes",
//  "album":"Meddle",
//  "artist": "Pink Floyd",
//  "artwork":[{"sizes":"128x128",
//              "src":"https://lh3.googleusercontent.com/p2_pHFA7u4uxGvEYoKvhiyxLDUCxPxJCMwRQLVMAMs4FF5lxb0hcVAa6iJY4UvMjrSiAwM6HiqXzyy4=w128-h128-l90-rj",
//              "type":"image/jpeg"}]}
pub fn create_media_indexer() -> FlatJsonIndexer {
    FlatJsonIndexer::new("application/x-media+json", &["title", "album", "artist"])
}
