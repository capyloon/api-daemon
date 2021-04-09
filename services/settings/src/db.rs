/// DB interface for the Settings
use crate::generated::common::*;
use common::observers::ObserverTracker;
use common::traits::DispatcherId;
use common::JsonValue;
use log::{error, info};
use rusqlite::{named_params, params, Connection, NO_PARAMS};
use serde_json::Value;
use sqlite_utils::{DatabaseUpgrader, SqliteDb};
use thiserror::Error;

#[cfg(not(target_os = "android"))]
const DB_PATH: &str = "./settings.sqlite";
#[cfg(target_os = "android")]
const DB_PATH: &str = "/data/local/service/api-daemon/settings.sqlite";

const TABLE_NAME: &str = "settings";

#[derive(Error, Debug)]
pub enum Error {
    #[error("SQlite error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Can't import invalid Json")]
    InvalidImport,
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Error::Sqlite(e1), Error::Sqlite(e2)) => e1 == e2,
            (Error::InvalidImport, Error::InvalidImport) => true,
            (..) => false,
        }
    }
}

pub struct SettingsSchemaManager {}

impl DatabaseUpgrader for SettingsSchemaManager {
    fn upgrade(&mut self, from: u32, to: u32, connection: &mut Connection) -> bool {
        // We only support version 1 currently.
        if !(from == 0 && to == 1) {
            return false;
        }

        connection
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS {} ( name TEXT UNIQUE, value TEXT)",
                    TABLE_NAME
                ),
                NO_PARAMS,
            )
            .is_ok()
    }
}

// The observers from other api-daemon services
pub trait DbObserver {
    fn callback(&self, name: &str, value: &JsonValue);
}

pub enum ObserverType {
    Proxy(SettingObserverProxy),
    FuncPtr(Box<dyn DbObserver + Sync + Send>),
}

pub struct SettingsDb {
    // A handle to the database.
    db: SqliteDb,
    // Handle to the event broadcaster to fire events when changing settings.
    event_broadcaster: SettingsFactoryEventBroadcaster,
    // The set of observers we may call. They are keyed on the setting name to
    // not slow down lookup when settings changes, even if that makes observer
    // removal slower.
    observers: ObserverTracker<String, ObserverType>,
}

impl SettingsDb {
    pub fn new(event_broadcaster: SettingsFactoryEventBroadcaster) -> Self {
        // TODO: manage error opening the db.
        let db = SqliteDb::open(DB_PATH, &mut SettingsSchemaManager {}, 1).unwrap();
        if let Err(err) = db.enable_wal() {
            error!("Failed to enable WAL mode on settings db: {}", err);
        }

        let mut settings_db = Self {
            db,
            event_broadcaster,
            observers: ObserverTracker::default(),
        };

        // Merge default settings.
        {
            #[cfg(target_os = "android")]
            let defaults_path: &str = "/system/b2g/defaults/settings.json";
            #[cfg(not(target_os = "android"))]
            let defaults_path =
                std::env::var("DEFAULT_SETTINGS").unwrap_or_else(|_| "".to_string());

            if !defaults_path.is_empty() {
                match std::fs::File::open(&defaults_path) {
                    Ok(defaults_file) => match serde_json::from_reader(defaults_file) {
                        Ok(json) => match settings_db.merge_json(&json) {
                            Ok(count) => info!("Imported {} new settings", count),
                            Err(err) => error!("Failed to import new settings: {}", err),
                        },
                        Err(err) => error!("Failed to serde_json::from_reader error {:?}", err),
                    },
                    Err(err) => error!("Failed to open file {} error {:?}.", defaults_path, err),
                }
            }
        }

        settings_db
    }

    pub fn add_dispatcher(&mut self, dispatcher: &SettingsFactoryEventDispatcher) -> DispatcherId {
        self.event_broadcaster.add(dispatcher)
    }

    pub fn remove_dispatcher(&mut self, id: DispatcherId) {
        self.event_broadcaster.remove(id)
    }

    pub fn add_observer(&mut self, name: &str, observer: ObserverType) -> DispatcherId {
        self.observers.add(name.into(), observer)
    }

    pub fn remove_observer(&mut self, name: &str, id: DispatcherId) -> bool {
        self.observers.remove(&name.into(), id)
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.db
            .connection()
            .execute(&format!("DELETE FROM {}", TABLE_NAME), params![])?;

        Ok(())
    }

    pub fn set(&mut self, settings: &[SettingInfo]) -> Result<(), Error> {
        let tx = self.db.mut_connection().transaction()?;
        {
            let mut stmt_del =
                tx.prepare(&format!("DELETE FROM {} WHERE name=:name", TABLE_NAME))?;
            let mut stmt_ins = tx.prepare(&format!(
                "INSERT OR REPLACE INTO {}(name, value) VALUES(:name, :value)",
                TABLE_NAME
            ))?;

            for setting_info in settings {
                let value = &*setting_info.value;
                if value == &Value::Null {
                    // setting_info.value is parsed from JSON.parse(object).
                    // If setting_info.value is empty string, it means that object
                    // is undefined, then delete the name from table.
                    stmt_del.execute_named(named_params! {":name": setting_info.name})?;
                } else {
                    // Store the string representation
                    // TODO: check if we should use SQlite json support.
                    let serialized = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
                    stmt_ins.execute_named(
                        named_params! {":name": setting_info.name, ":value": serialized},
                    )?;
                }

                // Dispatch a change event for this setting.
                self.event_broadcaster
                    .broadcast_change(setting_info.clone());

                // If we have observers for this setting, call their callback.
                self.observers.for_each(&setting_info.name, |obs, _id| {
                    match obs {
                        ObserverType::Proxy(cb) => {
                            cb.callback(setting_info.clone());
                        }
                        ObserverType::FuncPtr(cb) => {
                            cb.callback(&setting_info.name, &setting_info.value);
                        }
                    }
                });
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<JsonValue, Error> {
        let mut stmt = self.db.connection().prepare(&format!(
            "SELECT value FROM {} WHERE name=:name",
            TABLE_NAME
        ))?;

        let string: String = stmt.query_row_named(named_params! {":name": name}, |r| r.get(0))?;
        Ok(serde_json::from_str(&string).unwrap_or(Value::Null).into())
    }

    pub fn get_batch(&self, names: &[String]) -> Result<Vec<SettingInfo>, Error> {
        let mut result: Vec<SettingInfo> = Vec::new();

        if names.is_empty() {
            return Ok(result);
        }

        let mut stmt = self.db.connection().prepare(&format!(
            "SELECT name, value FROM {} WHERE name in ({}?)",
            TABLE_NAME,
            "?, ".repeat(names.len() - 1)
        ))?;

        let mut rows = stmt.query(names)?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(0).unwrap();
            let value: String = row.get(1).unwrap();

            result.push(SettingInfo {
                name,
                value: serde_json::from_str(&value).unwrap_or(Value::Null).into(),
            });
        }

        Ok(result)
    }
    // Import all the new value pairs from the json object.
    // Returns the number of inserted settings.
    pub fn merge_json(&mut self, json: &Value) -> Result<usize, Error> {
        if let Value::Object(map) = json {
            // Turns the map into a [SettingInfo] to insert: we only insert
            // setttings that are new.
            let settings: Vec<SettingInfo> = map
                .iter()
                .filter_map(|(name, value)| {
                    if self.get(name).is_err() {
                        Some(SettingInfo {
                            name: name.clone(),
                            value: JsonValue::from((*value).clone()),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            self.set(&settings)?;
            Ok(settings.len())
        } else {
            Err(Error::InvalidImport)
        }
    }

    // Display
    pub fn log(&self) {
        let count = self
            .db
            .connection()
            .query_row(
                &format!("SELECT count(*) FROM {}", TABLE_NAME),
                NO_PARAMS,
                |row| row.get(0),
            )
            .unwrap_or(0);
        info!("  {} settings in db.", count);

        info!(
            "  {} registered observers ({} keys).",
            self.observers.count(),
            self.observers.key_count()
        );

        self.event_broadcaster.log();
    }
}

#[test]
fn import_settings() {
    let mut db = SettingsDb::new(SettingsFactoryEventBroadcaster::default());
    assert!(db.clear().is_ok());
    let settings =
        serde_json::from_reader(std::fs::File::open("./test-fixtures/settings.json").unwrap())
            .unwrap();
    assert_eq!(db.merge_json(&settings).unwrap(), 295);
    assert_eq!(
        *db.get("app.update.battery-threshold.unplugged").unwrap(),
        Value::Number(25.into())
    );
    assert_eq!(
        *db.get("phone.dtmf.type").unwrap(),
        Value::String("long".into())
    );
    assert_eq!(*db.get("alarm.enabled").unwrap(), Value::Bool(false));

    // Import a new settings file with only one new setting.
    let settings =
        serde_json::from_reader(std::fs::File::open("./test-fixtures/settings_2.json").unwrap())
            .unwrap();
    assert_eq!(db.merge_json(&settings).unwrap(), 1);
    assert_eq!(
        *db.get("a.new.setting").unwrap(),
        Value::String("Hello World!".into())
    );

    let mut values = db
        .get_batch(&vec![
            String::from("app.update.battery-threshold.unplugged"),
            String::from("phone.dtmf.type"),
            String::from("alarm.enabled"),
        ])
        .unwrap();
    values.sort_by(|a, b| (a.name).cmp(&b.name));

    assert_eq!(values.len(), 3);
    assert_eq!(values[0].name, "alarm.enabled");
    assert_eq!(values[1].name, "app.update.battery-threshold.unplugged");
    assert_eq!(values[2].name, "phone.dtmf.type");
    assert_eq!(*values[0].value, Value::Bool(false));
    assert_eq!(*values[1].value, Value::Number(25.into()));
    assert_eq!(*values[2].value, Value::String("long".into()));
}
