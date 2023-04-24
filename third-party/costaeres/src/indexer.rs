/// Indexers for recognized mime types.
use crate::common::{ResourceMetadata, TransactionResult, Variant};
use crate::fts::Fts;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

#[async_trait(?Send)]
pub trait Indexer {
    async fn index<'c>(
        &self,
        meta: &ResourceMetadata,
        variant: &mut Variant,
        fts: &Fts,
        mut tx: Transaction<'c, Sqlite>,
    ) -> TransactionResult<'c>;
}

#[allow(clippy::upper_case_acronyms)]
type OVS = Option<Vec<String>>;

// A generic indexer for flat Json data structures.
// Indexed properties are strings and string arrays members.
pub struct FlatJsonIndexer {
    fields: Vec<String>,
    mime_type: String,
    #[allow(clippy::type_complexity)]
    custom_fn: Option<Box<dyn Fn(&str, &str) -> OVS + Send + Sync>>,
}

impl FlatJsonIndexer {
    #[allow(clippy::type_complexity)]
    pub fn new(
        mime_type: &str,
        fields: &[&str],
        custom_fn: Option<Box<dyn Fn(&str, &str) -> OVS + Send + Sync>>,
    ) -> Self {
        Self {
            fields: fields.iter().map(|e| (*e).to_owned()).collect(),
            mime_type: mime_type.into(),
            custom_fn,
        }
    }
}

#[async_trait(?Send)]
impl Indexer for FlatJsonIndexer {
    async fn index<'c>(
        &self,
        meta: &ResourceMetadata,
        variant: &mut Variant,
        fts: &Fts,
        mut tx: Transaction<'c, Sqlite>,
    ) -> TransactionResult<'c> {
        // 0. Filter by mime type.
        if self.mime_type != variant.metadata.mime_type() {
            return Ok(tx);
        }

        macro_rules! indexme {
            ($field:expr, $text:expr) => {
                if let Some(func) = &self.custom_fn {
                    if let Some(custom_text) = func($field, $text) {
                        for item in custom_text {
                            tx = fts
                                .add_text(&meta.id(), &variant.metadata.name(), &item, tx)
                                .await?;
                        }
                    }
                } else {
                    tx = fts
                        .add_text(&meta.id(), &variant.metadata.name(), $text, tx)
                        .await?;
                }
            };
        }

        let content = &mut variant.reader;

        // 1. Read the content as json.
        content.seek(SeekFrom::Start(0)).await?;
        let mut buffer = vec![];
        content.read_to_end(&mut buffer).await?;
        let v: Value = serde_json::from_slice(&buffer)?;

        // 2. Index each available field.
        for field in &self.fields {
            match v.get(field) {
                Some(Value::String(text)) => {
                    indexme!(field, text);
                }
                Some(Value::Array(array)) => {
                    for item in array {
                        if let Value::String(text) = item {
                            indexme!(field, text);
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
    FlatJsonIndexer::new("application/x-places+json", &["url", "title"], None)
}

// Indexer for the content of a "Contacts" object.
// This is a json value with the following format:
// { name: "...", phone: "[...]", email: "[...]" }
fn custom_contact_index(field: &str, text: &str) -> OVS {
    if text.is_empty() {
        None
    } else if field == "name" {
        Some(vec![
            text.to_owned(),
            format!("^^^^{}", text.chars().next().unwrap()),
        ])
    } else {
        Some(vec![text.to_owned()])
    }
}

pub fn create_contacts_indexer() -> FlatJsonIndexer {
    FlatJsonIndexer::new(
        "application/x-contact+json",
        &["name", "phone", "email"],
        Some(Box::new(custom_contact_index)),
    )
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
    FlatJsonIndexer::new(
        "application/x-media+json",
        &["title", "album", "artist"],
        None,
    )
}
