/// A naive implementation of Full Text Search.
/// The goal is to provide substring matching to object names and tags.
///
/// Using a simple SQlite table (ResourceId, ngram) which makes it easy to
/// manage object removal at the expense of disk space usage and query performance.
/// TODO: switch to a Key Value store (eg. Sled) instead, or a fts engine like Sonic.
use crate::common::{IdFrec, ResourceId, ResourceStoreError, TransactionResult};
use crate::timer::Timer;
use sqlx::{Sqlite, SqlitePool, Transaction};

pub struct Fts {
    db_pool: SqlitePool,
}

impl Fts {
    pub fn new(pool: &SqlitePool) -> Self {
        Self {
            db_pool: pool.clone(),
        }
    }

    pub async fn remove_text<'c>(
        &self,
        id: &ResourceId,
        variant: Option<&str>,
        mut tx: Transaction<'c, Sqlite>,
    ) -> TransactionResult<'c> {
        if let Some(v) = variant {
            sqlx::query!("DELETE FROM fts WHERE id = ? and variant = ?", id, v)
                .execute(&mut tx)
                .await?;
        } else {
            sqlx::query!("DELETE FROM fts WHERE id = ?", id)
                .execute(&mut tx)
                .await?;
        }

        Ok(tx)
    }

    pub async fn add_text<'c>(
        &self,
        id: &ResourceId,
        variant: &str,
        text: &str,
        mut tx: Transaction<'c, Sqlite>,
    ) -> TransactionResult<'c> {
        // Remove diacritics since the trigram tokenizer of SQlite doesn't have this option.
        let content = secular::lower_lay_string(text);

        sqlx::query!(
            "INSERT INTO fts ( id, variant, content ) VALUES ( ?, ?, ? )",
            id,
            variant,
            content
        )
        .execute(&mut tx)
        .await?;

        Ok(tx)
    }

    pub async fn search(
        &self,
        text: &str,
        tag: Option<String>,
    ) -> Result<Vec<IdFrec>, ResourceStoreError> {
        let _timer = Timer::start(&format!("Fts::search {text} {tag:?}"));

        let mut tx = self.db_pool.begin().await?;

        let search = format!("%{}%", secular::lower_lay_string(text));

        let records: Vec<IdFrec> =
            match tag {
                None => sqlx::query_as(
                    r#"SELECT resources.id, frecency(resources.scorer) AS frecency FROM resources
                        JOIN fts
                        WHERE fts.id = resources.id
                        AND fts.content LIKE ?
                        ORDER BY frecency DESC LIMIT 100"#,
                )
                .bind(&search)
                .fetch_all(&mut tx)
                .await?,
                Some(ref tag) => sqlx::query_as(
                    r#"SELECT resources.id, frecency(resources.scorer) AS frecency FROM resources
                        JOIN fts, tags
                        WHERE tags.tag = ?
                        AND fts.id = resources.id AND tags.id = resources.id
                        AND fts.content LIKE ?
                        ORDER BY frecency DESC LIMIT 100"#,
                )
                .bind(tag)
                .bind(&search)
                .fetch_all(&mut tx)
                .await?,
            };

        // Filter out duplicates.
        let mut seen = std::collections::HashSet::new();
        let mut results = vec![];
        for rec in records {
            if !seen.contains(&rec.id) {
                results.push(rec.clone());
                seen.insert(rec.id);
            }
        }
        Ok(results)
    }
}
