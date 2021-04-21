//! Rusqlite wrapper that adds support for version management.

use log::{error, info};
use rusqlite::{Connection, OpenFlags, NO_PARAMS};
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
    fn upgrade(&mut self, from: u32, to: u32, connection: &mut Connection) -> bool;
}

pub struct SqliteDb {
    connection: Connection,
}

impl SqliteDb {
    /// Opens a database targetting a version number. The schema upgrader will be
    /// called as needed if the current version doesn't match the targetted one.
    pub fn open_with_flags<P: AsRef<Path>, U: DatabaseUpgrader>(
        path: P,
        upgrader: &mut U,
        version: u32,
        flags: OpenFlags,
    ) -> Result<Self, SqliteDbError> {
        let mut connection = Connection::open_with_flags(&path, flags)?;

        // Get the current version.
        let current_version: u32 = connection.query_row(
            "SELECT user_version FROM pragma_user_version",
            NO_PARAMS,
            |r| r.get(0),
        )?;

        info!(
            "Current db {:?} version is {}, requested version is {}",
            path.as_ref(),
            current_version,
            version
        );

        // Downgrades are not supported.
        if current_version > version {
            return Err(SqliteDbError::SchemaUpgrade(current_version, version));
        }

        if current_version < version && !upgrader.upgrade(current_version, version, &mut connection)
        {
            return Err(SqliteDbError::SchemaUpgrade(current_version, version));
        }

        // The upgrade went fine, so we can set the new version number.
        if let Err(err) = connection.pragma_update(None, "user_version", &version) {
            error!(
                "Failed to update user_version in {:?}: {}",
                path.as_ref(),
                err
            );
            return Err(err.into());
        }

        Ok(SqliteDb { connection })
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
            .pragma_update(None, "journal_mode", &"WAL".to_string())?;
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

#[test]
fn upgrader_test() {
    use rusqlite::named_params;
    #[derive(Default)]
    struct OneShotUpgrader {
        upgraded: bool,
    };
    impl DatabaseUpgrader for OneShotUpgrader {
        fn upgrade(&mut self, from: u32, to: u32, connection: &mut Connection) -> bool {
            if !self.upgraded {
                assert_eq!((from, to), (0, 1));
                self.upgraded = true;
                // Create a basic schema.
                connection
                    .execute(
                        "CREATE TABLE IF NOT EXISTS test ( name TEXT UNIQUE, value TEXT)",
                        NO_PARAMS,
                    )
                    .unwrap();
            } else {
                assert!(false, "We should only upgrade this db once!");
            }
            true
        }
    }

    let mut upgrader = OneShotUpgrader::default();

    {
        // First db creation, will create the schema.
        let db = SqliteDb::open("./test-db.sqlite", &mut upgrader, 1).unwrap();
        db.enable_wal().unwrap();
    }

    {
        // Second db creation with the same version, the ugrader should not run.
        let mut db = SqliteDb::open("./test-db.sqlite", &mut upgrader, 1).unwrap();

        let connection = db.mut_connection();

        // We start with an empty database.
        let row_count: u32 = connection
            .query_row("SELECT COUNT(*) FROM test", NO_PARAMS, |r| r.get(0))
            .unwrap();
        assert_eq!(row_count, 0);

        // Add a row.
        let tx = connection.transaction().unwrap();
        {
            let mut stmt_ins = tx
                .prepare("INSERT OR REPLACE INTO test(name, value) VALUES(:name, :value)")
                .unwrap();

            stmt_ins
                .execute_named(named_params! {":name": "foo", ":value": "bar"})
                .unwrap();
        }
        tx.commit().unwrap();

        // Check that we have a row now.
        let row_count: u32 = connection
            .query_row("SELECT COUNT(*) FROM test", NO_PARAMS, |r| r.get(0))
            .unwrap();
        assert_eq!(row_count, 1);
    }

    let _ = std::fs::remove_file("./test-db.sqlite").unwrap();
}
