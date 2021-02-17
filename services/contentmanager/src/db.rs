/// DB interface for the Settings
use crate::generated::common::*;
use common::traits::DispatcherId;
use log::{error, info};
use rusqlite::{named_params, params, Connection, NO_PARAMS};
use sqlite_utils::{DatabaseUpgrader, SqliteDb};
use std::collections::HashMap;
use thiserror::Error;

#[cfg(not(target_os = "android"))]
const DB_PATH: &str = "./vfs.sqlite";
#[cfg(target_os = "android")]
const DB_PATH: &str = "/data/local/service/api-daemon/vfs.sqlite";

const TABLE_NAME: &str = "vfs";

#[derive(Error, Debug)]
pub enum Error {
    #[error("SQlite error")]
    Sqlite(#[from] rusqlite::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Error::Sqlite(e1), Error::Sqlite(e2)) => e1 == e2,
            (..) => false,
        }
    }
}

pub struct VfsSchemaManager {}

impl DatabaseUpgrader for VfsSchemaManager {
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
    fn callback(&self, id: i64, change: &ObjectModification);
}

pub enum ObserverType {
    Proxy(ModificationObserverProxy),
    FuncPtr(Box<dyn DbObserver + Sync + Send>),
}

pub struct VfsDb {
    // Current id that we hand out when an observer is registered.
    id: DispatcherId,
    // A handle to the database.
    db: SqliteDb,
    // Handle to the event broadcaster to fire events when changes happen.
    event_broadcaster: ContentStoreEventBroadcaster,
    // The set of observers we may call. They are keyed on the node id.
    observers: HashMap<i64, Vec<(ObserverType, DispatcherId)>>,
}

impl Default for VfsDb {
    fn default() -> Self {
        VfsDb::new(ContentStoreEventBroadcaster::default())
    }
}

impl VfsDb {
    pub fn new(event_broadcaster: ContentStoreEventBroadcaster) -> Self {
        // TODO: manage error opening the db.
        let db = SqliteDb::open(DB_PATH, &mut VfsSchemaManager {}, 1).unwrap();
        if let Err(err) = db.enable_wal() {
            error!("Failed to enable WAL mode on vfs db: {}", err);
        }

        let mut vfs_db = Self {
            id: 0,
            db,
            event_broadcaster,
            observers: HashMap::new(),
        };

        // If the database is empty, create the initial root node.

        vfs_db
    }

    pub fn add_dispatcher(&mut self, dispatcher: &ContentStoreEventDispatcher) -> DispatcherId {
        self.event_broadcaster.add(dispatcher)
    }

    pub fn remove_dispatcher(&mut self, id: DispatcherId) {
        self.event_broadcaster.remove(id)
    }

    pub fn add_observer(&mut self, id: i64, observer: ObserverType) -> DispatcherId {
        self.id += 1;

        match self.observers.get_mut(&id) {
            Some(observers) => {
                observers.push((observer, self.id));
            }
            None => {
                let init = vec![(observer, self.id)];
                self.observers.insert(id, init);
            }
        }

        self.id
    }

    pub fn remove_observer(&mut self, id: i64, dispatcher: DispatcherId) {
        for (key, entry) in self.observers.iter_mut() {
            if id != *key {
                continue;
            }
            // Remove the vector items that have the matching dispatcher.
            // Note: Once it's in stable Rustc, we could simply use:
            // entry.drain_filter(|item| item.1 == dispatcher);
            let mut i = 0;
            while i != entry.len() {
                if entry[i].1 == dispatcher {
                    entry.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.db
            .connection()
            .execute(&format!("DELETE FROM {}", TABLE_NAME), params![])?;

        Ok(())
    }

    // pub fn set(&mut self, settings: &[SettingInfo]) -> Result<(), Error> {
    //     let tx = self.db.mut_connection().transaction()?;
    //     {
    //         let mut stmt_del =
    //             tx.prepare(&format!("DELETE FROM {} WHERE name=:name", TABLE_NAME))?;
    //         let mut stmt_ins = tx.prepare(&format!(
    //             "INSERT OR REPLACE INTO {}(name, value) VALUES(:name, :value)",
    //             TABLE_NAME
    //         ))?;

    //         for setting_info in settings {
    //             let value = &*setting_info.value;
    //             if value == &Value::Null {
    //                 // setting_info.value is parsed from JSON.parse(object).
    //                 // If setting_info.value is empty string, it means that object
    //                 // is undefined, then delete the name from table.
    //                 stmt_del.execute_named(named_params! {":name": setting_info.name})?;
    //             } else {
    //                 // Store the string representation
    //                 // TODO: check if we should use SQlite json support.
    //                 let serialized = serde_json::to_string(&value).unwrap_or_else(|_| "{}".into());
    //                 stmt_ins.execute_named(
    //                     named_params! {":name": setting_info.name, ":value": serialized},
    //                 )?;
    //             }

    //             // Dispatch a change event for this setting.
    //             self.event_broadcaster
    //                 .broadcast_change(setting_info.clone());

    //             // If we have observers for this setting, call their callback.
    //             if let Some(observers) = self.observers.get_mut(&setting_info.name) {
    //                 for observer in observers {
    //                     let (obs, _) = observer;
    //                     match obs {
    //                         ObserverType::Proxy(cb) => {
    //                             cb.callback(setting_info.clone());
    //                         }
    //                         ObserverType::FuncPtr(cb) => {
    //                             cb.callback(&setting_info.name, &setting_info.value);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     tx.commit()?;
    //     Ok(())
    // }

    // pub fn get(&self, name: &str) -> Result<JsonValue, Error> {
    //     let mut stmt = self.db.connection().prepare(&format!(
    //         "SELECT value FROM {} WHERE name=:name",
    //         TABLE_NAME
    //     ))?;

    //     let string: String = stmt.query_row_named(named_params! {":name": name}, |r| r.get(0))?;
    //     Ok(serde_json::from_str(&string).unwrap_or(Value::Null).into())
    // }
}

// #[test]
// fn import_settings() {
//     let mut db = VfsDb::new(SettingsFactoryEventBroadcaster::default());
//     assert!(db.clear().is_ok());
//     let settings =
//         serde_json::from_reader(std::fs::File::open("./test-fixtures/settings.json").unwrap())
//             .unwrap();
//     assert_eq!(db.merge_json(&settings).unwrap(), 295);
//     assert_eq!(
//         *db.get("app.update.battery-threshold.unplugged").unwrap(),
//         Value::Number(25.into())
//     );
//     assert_eq!(
//         *db.get("phone.dtmf.type").unwrap(),
//         Value::String("long".into())
//     );
//     assert_eq!(*db.get("alarm.enabled").unwrap(), Value::Bool(false));

//     // Import a new settings file with only one new setting.
//     let settings =
//         serde_json::from_reader(std::fs::File::open("./test-fixtures/settings_2.json").unwrap())
//             .unwrap();
//     assert_eq!(db.merge_json(&settings).unwrap(), 1);
//     assert_eq!(
//         *db.get("a.new.setting").unwrap(),
//         Value::String("Hello World!".into())
//     );

//     let mut values = db
//         .get_batch(&vec![
//             String::from("app.update.battery-threshold.unplugged"),
//             String::from("phone.dtmf.type"),
//             String::from("alarm.enabled"),
//         ])
//         .unwrap();
//     values.sort_by(|a, b| (a.name).cmp(&b.name));

//     assert_eq!(values.len(), 3);
//     assert_eq!(values[0].name, "alarm.enabled");
//     assert_eq!(values[1].name, "app.update.battery-threshold.unplugged");
//     assert_eq!(values[2].name, "phone.dtmf.type");
//     assert_eq!(*values[0].value, Value::Bool(false));
//     assert_eq!(*values[1].value, Value::Number(25.into()));
//     assert_eq!(*values[2].value, Value::String("long".into()));
// }
