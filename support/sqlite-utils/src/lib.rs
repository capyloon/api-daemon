//! Rusqlite wrapper that adds support for version management.

use log::{error, info};
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqliteDbError {
    #[error("Rusqlite Error")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Error upgrading db schema from version `{0}` to version `{1}`")]
    SchemaUpgrade(u32, u32),
}

pub trait DatabaseUpgrader {
    fn upgrade(&mut self, from: u32, to: u32, transaction: &Transaction) -> bool;
}

pub struct SqliteDb {
    connection: Connection,
}

impl SqliteDb {
    /// Opens a database targetting a version number. The schema upgrader will be
    /// called as needed if the current version doesn't match the targeted one.
    fn open_with_flags_internal<P: AsRef<Path>, U: DatabaseUpgrader>(
        path: P,
        upgrader: &mut U,
        version: u32,
        flags: OpenFlags,
    ) -> Result<Self, SqliteDbError> {
        let mut connection = Connection::open_with_flags(&path, flags)?;

        // Get the current version.
        let current_version: u32 =
            connection.query_row("SELECT user_version FROM pragma_user_version", [], |r| {
                r.get(0)
            })?;

        info!(
            "Current db {:?} version is {}, requested version is {}",
            path.as_ref(),
            current_version,
            version
        );

        {
            // Create a scoped transaction to run the schema update steps and the pragma update.
            // The default drop behavior of Transaction is to rollback changes, so we
            // explicitely commit it once all the operations succeeded.
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

            // Downgrades are not supported.
            if current_version > version {
                return Err(SqliteDbError::SchemaUpgrade(current_version, version));
            }

            if current_version < version {
                if !upgrader.upgrade(current_version, version, &transaction) {
                    return Err(SqliteDbError::SchemaUpgrade(current_version, version));
                } else {
                    // The upgrade went fine, so we can set the new version number.
                    if let Err(err) = transaction.pragma_update(None, "user_version", version) {
                        error!(
                            "Failed to update user_version in {:?}: {}",
                            path.as_ref(),
                            err
                        );
                        return Err(err.into());
                    }
                }
            }

            transaction.commit()?;
        }

        Ok(SqliteDb { connection })
    }

    /// Opens a database targetting a version number. The schema upgrader will be
    /// called as needed if the current version doesn't match the targeted one.
    /// If the first run of the upgrader fails, we re-start the process with an
    /// empty database.
    pub fn open_with_flags<P: AsRef<Path>, U: DatabaseUpgrader>(
        path: P,
        upgrader: &mut U,
        version: u32,
        flags: OpenFlags,
    ) -> Result<Self, SqliteDbError> {
        match Self::open_with_flags_internal(&path, upgrader, version, flags) {
            Ok(db) => Ok(db),
            Err(err) => {
                error!(
                    "First database upgrade for {:?} failed: {}, retrying.",
                    path.as_ref(),
                    err
                );
                // First try failed. This can happen in case of fatal db corruption,
                // so we delete the file and try again.
                let _ = std::fs::remove_file(&path);
                Self::open_with_flags_internal(&path, upgrader, version, flags)
            }
        }
    }

    /// Open a database with the default flags.
    pub fn open<P: AsRef<Path>, U: DatabaseUpgrader>(
        path: P,
        upgrader: &mut U,
        version: u32,
    ) -> Result<Self, SqliteDbError> {
        SqliteDb::open_with_flags(path, upgrader, version, OpenFlags::default())
    }

    /// Enable WAL on this database (https://sqlite.org/wal.html).
    pub fn enable_wal(&self) -> Result<(), SqliteDbError> {
        self.connection
            .pragma_update(None, "journal_mode", "WAL".to_string())?;
        Ok(())
    }

    /// Hands out the underlying sqlite connection.
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Hands out a mutable version of the underlying sqlite connection.
    pub fn mut_connection(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default)]
    struct TestUpgrader {
        pub current_upgrades: u32,
    }

    impl DatabaseUpgrader for TestUpgrader {
        fn upgrade(&mut self, from: u32, to: u32, transaction: &Transaction) -> bool {
            assert_eq!((from, to), (0, 1));
            self.current_upgrades += 1;
            // Create a basic schema.
            transaction
                .execute("CREATE TABLE test ( name TEXT UNIQUE, value TEXT)", [])
                .is_ok()
        }
    }

    #[test]
    fn upgrader_test() {
        use rusqlite::named_params;

        let mut upgrader = TestUpgrader::default();

        {
            // First db creation, will create the schema.
            let db = SqliteDb::open("./test-db.sqlite", &mut upgrader, 1).unwrap();
            db.enable_wal().unwrap();
            assert_eq!(upgrader.current_upgrades, 1);
        }

        {
            // Second db creation with the same version, the ugrader should not run.
            let mut db = SqliteDb::open("./test-db.sqlite", &mut upgrader, 1).unwrap();

            assert_eq!(upgrader.current_upgrades, 1);

            let connection = db.mut_connection();

            // We start with an empty database.
            let row_count: u32 = connection
                .query_row("SELECT COUNT(*) FROM test", [], |r| r.get(0))
                .unwrap();
            assert_eq!(row_count, 0);

            // Add a row.
            let tx = connection.transaction().unwrap();
            {
                let mut stmt_ins = tx
                    .prepare("INSERT OR REPLACE INTO test(name, value) VALUES(:name, :value)")
                    .unwrap();

                stmt_ins
                    .execute(named_params! {":name": "foo", ":value": "bar"})
                    .unwrap();
            }
            tx.commit().unwrap();

            // Check that we have a row now.
            let row_count: u32 = connection
                .query_row("SELECT COUNT(*) FROM test", [], |r| r.get(0))
                .unwrap();
            assert_eq!(row_count, 1);
        }

        let _ = std::fs::remove_file("./test-db.sqlite").unwrap();
    }

    #[test]
    fn corrupted_db() {
        env_logger::init();

        // Copy the corrupted db since it will be "fixed".
        std::fs::copy(
            "./test-fixtures/settings-corrupted.sqlite",
            "./test-corrupted.sqlite",
        )
        .expect("Failed to copy corrupted db");

        {
            let mut upgrader = TestUpgrader::default();

            // The upgrade will run twice to "repair" the database.
            let _db = SqliteDb::open("./test-corrupted.sqlite", &mut upgrader, 1).unwrap();
            assert_eq!(upgrader.current_upgrades, 2);
        }

        // Database is now repaired, no further update step needed.
        let mut upgrader = TestUpgrader::default();

        let _db = SqliteDb::open("./test-corrupted.sqlite", &mut upgrader, 1).unwrap();
        assert_eq!(upgrader.current_upgrades, 0);

        let _ = std::fs::remove_file("./test-corrupted.sqlite").unwrap();
    }
}
