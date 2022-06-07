// Storage of DID identifiers.
// Currently using simple JSON storage, making sure we sync()
// the file after each write.
// The proper solution needs to use some secure storage, which
// will be platform dependent.

use crate::did::Did;
use crate::generated::common::Did as SidlDid;
use log::error;
use rusqlite::{named_params, Transaction};
use sqlite_utils::{DatabaseUpgrader, SqliteDb};
use std::fs::create_dir_all;
use std::path::Path;

static UPGRADE_0_1_SQL: [&str; 2] = [
    r#"CREATE TABLE IF NOT EXISTS dids (
    name TEXT UNIQUE,
    uri TEXT NOT NULL,
    pubkey TEXT NOT NULL,
    privkey TEXT NOT NULL)"#,
    r#"CREATE UNIQUE INDEX IF NOT EXISTS did_uri ON dids(uri)"#,
];

pub struct DwebSchemaManager {}

impl DatabaseUpgrader for DwebSchemaManager {
    fn upgrade(&mut self, from: u32, to: u32, transaction: &Transaction) -> bool {
        // We only support version 1 currently.
        if to != 1 {
            return false;
        }

        let mut current = from;

        macro_rules! execute_commands {
            ($from:expr, $cmds:expr) => {
                if current == $from && current < to {
                    for cmd in $cmds {
                        if let Err(err) = transaction.execute(cmd, []) {
                            error!("Upgrade step failure: {}", err);
                            return false;
                        }
                    }
                    current += 1;
                }
            };
        }
        // Upgrade from version 0.
        execute_commands!(0, &UPGRADE_0_1_SQL);

        // At the end, the current version should match the expected one.
        current == to
    }
}

pub struct DidStorage {
    // A handle to the database.
    db: SqliteDb,
}

impl DidStorage {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        // Create the repository if it doesn't exist.
        // Will panic if that fails.
        if let Err(err) = create_dir_all(&path) {
            panic!(
                "Failed to create dweb path {} at {}",
                path.as_ref().display(),
                err
            );
        }

        let path = path.as_ref().join("dweb.sqlite");
        let db = SqliteDb::open(&path, &mut DwebSchemaManager {}, 1).unwrap();
        if let Err(err) = db.enable_wal() {
            error!("Failed to enable WAL mode on dweb db: {}", err);
        }

        let mut res = Self { db };

        // Create a default super user.
        if res.did_count().unwrap_or(0) == 0 {
            let _ = res.add(&Did::superuser());
        }

        res
    }

    pub fn did_count(&self) -> Result<u32, rusqlite::Error> {
        let mut stmt = self
            .db
            .connection()
            .prepare(&format!("SELECT count(name) FROM dids"))?;

        let count = stmt.query_row([], |r| Ok(r.get_unwrap(0)))?;
        Ok(count)
    }

    pub fn add(&mut self, did: &Did) -> Result<bool, rusqlite::Error> {
        let mut stmt = self
            .db
            .connection()
            .prepare("INSERT INTO dids(name, uri, pubkey, privkey) VALUES(?, ?, ?, ?)")?;
        let size = stmt.execute(&[&did.name, &did.uri(), &did.pubkey_b64(), &did.privkey_b64()])?;
        if size > 0 {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn remove(&mut self, uri: &str) -> Result<bool, rusqlite::Error> {
        let mut stmt = self
            .db
            .connection()
            .prepare(&format!("DELETE FROM dids WHERE uri=:uri"))?;
        stmt.execute(named_params! {":uri": uri})?;

        Ok(true)
    }

    pub fn get_all(&self) -> Result<Vec<SidlDid>, rusqlite::Error> {
        let mut stmt = self
            .db
            .connection()
            .prepare("SELECT name, pubkey, privkey FROM dids")?;
        let mut rows = stmt.query([])?;
        let mut result = vec![];
        while let Some(row) = rows.next()? {
            if let Ok(did) = Did::from_row(row) {
                result.push(did.into());
            }
        }
        Ok(result)
    }

    pub fn by_name(&self, name: &str) -> Result<Option<Did>, rusqlite::Error> {
        let mut stmt = self.db.connection().prepare(&format!(
            "SELECT name, pubkey, privkey FROM dids WHERE name = ?",
        ))?;
        let mut rows = stmt.query(&[name])?;
        if let Some(row) = rows.next()? {
            if let Ok(did) = Did::from_row(row) {
                return Ok(Some(did.into()));
            }
        }

        Ok(None)
    }

    pub fn by_uri(&self, uri: &str) -> Result<Option<Did>, rusqlite::Error> {
        let mut stmt = self.db.connection().prepare(&format!(
            "SELECT name, pubkey, privkey FROM dids WHERE uri = ?",
        ))?;
        let mut rows = stmt.query(&[uri])?;
        if let Some(row) = rows.next()? {
            if let Ok(did) = Did::from_row(row) {
                return Ok(Some(did.into()));
            }
        }

        Ok(None)
    }
}
