/// SQlite storage for the app registry.
use crate::apps_item::AppsItem;
use crate::generated::common::*;
use log::{debug, error};
use rusqlite::types::*;
use rusqlite::{named_params, Row, Transaction};
use serde_json::Value;
use sqlite_utils::{DatabaseUpgrader, SqliteDb, SqliteDbError};
use std::path::Path;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Rusqlite error")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("SqliteDb error")]
    SqliteDb(#[from] SqliteDbError),
    #[error("FromSql error {0}")]
    FromSql(String),
}

pub struct RegistryDb {
    // A handle to the database.
    db: SqliteDb,
}

impl From<AppsStatus> for String {
    fn from(s: AppsStatus) -> String {
        match s {
            AppsStatus::Disabled => "Disabled".into(),
            AppsStatus::Enabled => "Enabled".into(),
        }
    }
}

impl FromSql for AppsStatus {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(bytes) => {
                let s = String::from_utf8_lossy(bytes);
                if s == "Disabled" {
                    Ok(AppsStatus::Disabled)
                } else if s == "Enabled" {
                    Ok(AppsStatus::Enabled)
                } else {
                    let error = Error::FromSql(format!("Invalid AppsStatus: {}", s));
                    Err(FromSqlError::Other(Box::new(error)))
                }
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl From<AppsInstallState> for String {
    fn from(a: AppsInstallState) -> String {
        match a {
            AppsInstallState::Installed => "Installed".into(),
            AppsInstallState::Installing => "Installing".into(),
            AppsInstallState::Pending => "Pending".into(),
        }
    }
}

impl FromSql for AppsInstallState {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(bytes) => {
                let s = String::from_utf8_lossy(bytes);
                if s == "Installed" {
                    Ok(AppsInstallState::Installed)
                } else if s == "Installing" {
                    Ok(AppsInstallState::Installing)
                } else if s == "Pending" {
                    Ok(AppsInstallState::Pending)
                } else {
                    let error = Error::FromSql(format!("Invalid AppsInstallState: {}", s));
                    Err(FromSqlError::Other(Box::new(error)))
                }
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl From<AppsUpdateState> for String {
    fn from(a: AppsUpdateState) -> String {
        match a {
            AppsUpdateState::Available => "Available".into(),
            AppsUpdateState::Downloading => "Downloading".into(),
            AppsUpdateState::Idle => "Idle".into(),
            AppsUpdateState::Pending => "Pending".into(),
            AppsUpdateState::Updating => "Updating".into(),
        }
    }
}

impl FromSql for AppsUpdateState {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(bytes) => {
                let s = String::from_utf8_lossy(bytes);
                if s == "Available" {
                    Ok(AppsUpdateState::Available)
                } else if s == "Downloading" {
                    Ok(AppsUpdateState::Downloading)
                } else if s == "Idle" {
                    Ok(AppsUpdateState::Idle)
                } else if s == "Pending" {
                    Ok(AppsUpdateState::Pending)
                } else if s == "Updating" {
                    Ok(AppsUpdateState::Updating)
                } else {
                    let error = Error::FromSql(format!("Invalid AppsUpdateState: {}", s));
                    Err(FromSqlError::Other(Box::new(error)))
                }
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

// Will store the fields of an AppItem:
// {
// "name": "system",
// "version": "",
// "removable": false,
// "manifest_url": "http://system.localhost/manifest.webmanifest",
// "update_manifest_url": "http://cached.localhost/appname/update.webmanifest",
// "update_url": "https://store.server/system/manifest.webmanifest",
// "preloaded": false,
// "status": "Enabled",
// "install_state": "Installed",
// "update_state": "Idle",
// "install_time": 1584670494752,
// "update_time": 1593708589477,
// "manifest_hash": "cce24c3687d93c1ee00815d575bf4e6d",
// "package_hash": "fe16801bcceb73d135fbd4ac297edc2d",
// "manifest_etag": "W/\"5417c9e27c1c32b6dc4adf8bffe0030848c60a4c071440159573507d109ff4b2\""
// },

pub struct AppsSchemaManager {}

static UPGRADE_0_1_SQL: [&str; 1] = [r#"CREATE TABLE IF NOT EXISTS apps (
                                        name TEXT NOT NULL,
                                        version TEXT,
                                        removable BOOL,
                                        manifest_url TEXT NOT NULL UNIQUE,
                                        update_manifest_url TEXT NOT NULL,
                                        update_url TEXT NOT NULL,
                                        preloaded BOOL,
                                        status TEXT NOT NULL,
                                        install_state TEXT NOT NULL,
                                        update_state TEXT NOT NULL,
                                        install_time INTEGER,
                                        update_time INTEGER,
                                        manifest_hash TEXT,
                                        package_hash TEXT)"#];

static UPGRADE_1_2_SQL: [&str; 1] = [r#"ALTER TABLE apps
                                        ADD COLUMN manifest_etag TEXT"#];

static UPGRADE_2_3_SQL: [&str; 1] = [r#"ALTER TABLE apps
                                        ADD COLUMN deeplinks TEXT"#];

impl DatabaseUpgrader for AppsSchemaManager {
    fn upgrade(&mut self, from: u32, to: u32, connection: &Transaction) -> bool {
        // Support version 2 only.
        if to != 3 {
            return false;
        }

        let mut current = from;

        macro_rules! execute_commands {
            ($from:expr, $cmds:expr) => {
                if current == $from && current < to {
                    for cmd in $cmds {
                        if let Err(err) = connection.execute(cmd, []) {
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

        // Upgrade from version 1.
        // To be compatible with version 1 that has manifest_etag.
        if let Ok(stmt) = connection.prepare("SELECT * FROM apps") {
            if stmt.column_index("manifest_etag").is_err() {
                execute_commands!(1, &UPGRADE_1_2_SQL);
            } else {
                current += 1;
            }
        }

        // Upgrade from version 2.
        execute_commands!(2, &UPGRADE_2_3_SQL);

        // At the end, the current version should match the expected one.
        current == to
    }
}

// Converts a rusqlite row into a AppsItem.
fn row_to_apps_item(row: &Row) -> Result<AppsItem, rusqlite::Error> {
    let name: String = row.get("name")?;
    let version: String = row.get("version")?;
    let removable: bool = row.get("removable")?;
    let manifest_url: Url = row.get("manifest_url")?;
    let update_manifest_url: String = row.get("update_manifest_url")?;
    let update_url: String = row.get("update_url")?;
    let preloaded: bool = row.get("preloaded")?;
    let install_time: i64 = row.get("install_time")?;
    let update_time: i64 = row.get("update_time")?;
    let manifest_hash: String = row.get("manifest_hash")?;
    let package_hash: String = row.get("package_hash")?;
    let manifest_etag: Option<String> = row.get("manifest_etag").ok();
    let deeplinks: Option<Value> = row.get("deeplinks").ok();

    let mut item = AppsItem::new(&name, manifest_url);
    item.set_version(&version);
    item.set_removable(removable);
    item.set_update_manifest_url(Url::parse(&update_manifest_url).ok());
    item.set_update_url(Url::parse(&update_url).ok());
    item.set_preloaded(preloaded);
    item.set_status(row.get("status")?);
    item.set_install_state(row.get("install_state")?);
    item.set_update_state(row.get("update_state")?);
    item.set_install_time(install_time as _);
    item.set_update_time(update_time as _);
    item.set_manifest_hash(&manifest_hash);
    item.set_package_hash(&package_hash);
    item.set_manifest_etag(manifest_etag);
    item.set_deeplink_paths(deeplinks);
    Ok(item)
}

impl RegistryDb {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        // Open db with version 3.
        let db = SqliteDb::open(path, &mut AppsSchemaManager {}, 3)?;

        if let Err(err) = db.enable_wal() {
            error!("Failed to enable WAL mode on settings db: {}", err);
        }

        Ok(Self { db })
    }

    pub fn count(&self) -> Result<u32, Error> {
        debug!("RegistryDb::count");
        let mut stmt = self.db.connection().prepare("SELECT COUNT(*) FROM apps")?;

        let count = stmt.query_row([], |r| Ok(r.get_unwrap(0)))?;

        Ok(count)
    }

    pub fn add(&mut self, app: &AppsItem) -> Result<(), Error> {
        debug!("RegistryDb::add {}", app.get_name());
        debug!("  manifest_url is {:?}", app.get_manifest_url());
        debug!("  update_url is {:?}", app.get_update_url());

        macro_rules! url_not_null {
            ($url:expr) => {
                if let Some(url) = $url {
                    url.as_str()
                } else {
                    "".into()
                }
            };
        }

        let connection = self.db.mut_connection();
        let tx = connection.transaction()?;
        {
            let mut stmt_ins = tx.prepare(
                r#"INSERT OR REPLACE INTO apps (name,
                                     version,
                                     removable,
                                     manifest_url,
                                     update_manifest_url,
                                     update_url,
                                     preloaded,
                                     status,
                                     install_state,
                                     update_state,
                                     install_time,
                                     update_time,
                                     manifest_hash,
                                     package_hash,
                                     manifest_etag,
                                     deeplinks)
                             VALUES(:name,
                                    :version,
                                    :removable,
                                    :manifest_url,
                                    :update_manifest_url,
                                    :update_url,
                                    :preloaded,
                                    :status,
                                    :install_state,
                                    :update_state,
                                    :install_time,
                                    :update_time,
                                    :manifest_hash,
                                    :package_hash,
                                    :manifest_etag,
                                    :deeplinks)"#,
            )?;

            let status: String = app.get_status().into();
            let install_state: String = app.get_install_state().into();
            let update_state: String = app.get_update_state().into();
            stmt_ins.execute(named_params! {
                ":name": &app.get_name(),
                ":version": &app.get_version(),
                ":removable": &app.get_removable(),
                ":manifest_url": &app.get_manifest_url(),
                ":update_manifest_url": url_not_null!(&app.get_update_manifest_url()),
                ":update_url": url_not_null!(&app.get_update_url()),
                ":preloaded": &app.get_preloaded(),
                ":status": &status,
                ":install_state": &install_state,
                ":update_state": &update_state,
                ":install_time": &(app.get_install_time() as i64),
                ":update_time": &(app.get_update_time() as i64),
                ":manifest_hash": &app.get_manifest_hash(),
                ":package_hash": &app.get_package_hash(),
                ":manifest_etag": &app.get_manifest_etag().unwrap_or_else(|| "".into()),
                ":deeplinks": &app.get_deeplink_paths(),
            })?;
        }
        tx.commit()?;

        debug!("Success adding {}", app.get_name());

        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<AppsItem>, Error> {
        debug!("RegistryDb::get_all");
        let mut statement = self.db.connection().prepare("SELECT * FROM apps")?;
        let rows = statement.query_map([], row_to_apps_item)?;
        let results = rows
            .filter_map(|item| {
                if let Ok(app_item) = item {
                    Some(app_item)
                } else {
                    None
                }
            })
            .collect();
        Ok(results)
    }

    pub fn get_by_manifest_url(&self, manifest_url: &Url) -> Result<AppsItem, Error> {
        debug!("RegistryDb::get_by_manifest_url {}", manifest_url.as_str());
        let mut stmt = self
            .db
            .connection()
            .prepare("SELECT * FROM apps WHERE manifest_url=:manifest_url")?;

        stmt.query_row(
            named_params! {":manifest_url": manifest_url.as_str()},
            |r| Ok(row_to_apps_item(r).map_err(|e| e.into())),
        )?
    }

    pub fn get_by_update_url(&self, update_url: &Url) -> Result<AppsItem, Error> {
        debug!("RegistryDb::get_by_update_url {}", update_url.as_str());
        let mut stmt = self
            .db
            .connection()
            .prepare("SELECT * FROM apps WHERE update_url=:update_url")?;

        stmt.query_row(named_params! {":update_url": update_url.as_str()}, |r| {
            Ok(row_to_apps_item(r).map_err(|e| e.into()))
        })?
    }

    pub fn get_first_by_name(&self, name: &str) -> Result<AppsItem, Error> {
        debug!("RegistryDb::get_first_by_name {}", name);
        let mut stmt = self
            .db
            .connection()
            .prepare("SELECT * FROM apps WHERE name=:name")?;

        stmt.query_row(named_params! {":name": name}, |r| {
            Ok(row_to_apps_item(r).map_err(|e| e.into()))
        })?
    }

    pub fn remove_by_manifest_url(&mut self, manifest_url: &Url) -> Result<(), Error> {
        debug!(
            "RegistryDb::remove_by_manifest_url {}",
            manifest_url.as_str()
        );
        let connection = self.db.mut_connection();
        let tx = connection.transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM apps WHERE manifest_url=:manifest_url")?;
            stmt.execute(named_params! {":manifest_url": manifest_url.as_str()})?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn update_status(&mut self, manifest_url: &Url, status: AppsStatus) -> Result<(), Error> {
        let status: String = status.into();
        debug!(
            "RegistryDb::update_status {} for {}",
            status,
            manifest_url.as_str()
        );
        let connection = self.db.mut_connection();
        let tx = connection.transaction()?;
        {
            let mut stmt = tx.prepare("UPDATE apps SET status = ?1 WHERE manifest_url = ?2")?;
            stmt.execute([&status, manifest_url.as_str()])?;
        }
        tx.commit()?;
        Ok(())
    }
}
