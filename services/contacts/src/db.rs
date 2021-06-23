use crate::cursor::ContactDbCursor;
/// DB interface for the Contacts
use crate::generated::common::*;
use crate::preload::*;
use android_utils::{AndroidProperties, PropertyGetter};
use common::traits::DispatcherId;
use common::SystemTime;
use log::{debug, error};
use phonenumber::country::Id;
use phonenumber::Mode;
use rusqlite::{Connection, Row, Statement, NO_PARAMS};
use sqlite_utils::{DatabaseUpgrader, SqliteDb};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};
use thiserror::Error;
use threadpool::ThreadPool;
use uuid::Uuid;

#[cfg(not(target_os = "android"))]
const DB_PATH: &str = "./contacts.sqlite";
#[cfg(target_os = "android")]
const DB_PATH: &str = "/data/local/service/api-daemon/contacts.sqlite";

const MIN_MATCH_DIGITS: usize = 7;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SQlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serde JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid FilterOption error")]
    InvalidFilterOption(String),
    #[error("Invalid contact id error")]
    InvalidContactId(String),
    #[error("Ice position already used")]
    IcePositionUsed(String),
    #[error("Read File Error")]
    File(String),
    #[error("Parse Time Error: {0}")]
    ParseTime(#[from] chrono::format::ParseError),
    #[error("Time Error")]
    Time(String),
}

pub struct ContactsSchemaManager {}

static UPGRADE_0_1_SQL: [&str; 14] = [
    // Main table holding main data of contact.
    r#"CREATE TABLE IF NOT EXISTS contact_main (
        contact_id   TEXT    NOT NULL PRIMARY KEY,
        name         TEXT    DEFAULT (''),
        family_name  TEXT    DEFAULT (''),
        given_name   TEXT    DEFAULT (''),
        tel_number   TEXT    DEFAULT (''),
        tel_json     TEXT    DEFAULT (''),
        email        TEXT    DEFAULT (''),
        email_json   TEXT    DEFAULT (''),
        photo_type   TEXT    DEFAULT (''),
        photo_blob   BLOB    DEFAULT (x''),
        published    INTEGER DEFAULT (0),
        updated      INTEGER DEFAULT (0),
        bday         INTEGER DEFAULT (0),
        anniversary  INTEGER DEFAULT (0),
        category     TEXT    DEFAULT (''),
        category_json TEXT   DEFAULT ('')
    )"#,
    r#"CREATE INDEX IF NOT EXISTS idx_name ON contact_main(name)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_famil_name ON contact_main(family_name)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_given_name ON contact_main(given_name)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_tel_number ON contact_main(tel_number)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_email ON contact_main(email)"#,
    r#"CREATE INDEX IF NOT EXISTS idx_category ON contact_main(category)"#,
    r#"CREATE TABLE IF NOT EXISTS contact_additional (
        contact_id TEXT NOT NULL,
        data_type TEXT NOT NULL,
        value TEXT DEFAULT '',
        FOREIGN KEY(contact_id) REFERENCES contact_main(contact_id) ON DELETE CASCADE
    )"#,
    r#"CREATE INDEX IF NOT EXISTS idx_additional ON contact_additional(contact_id)"#,
    r#"CREATE TABLE IF NOT EXISTS blocked_numbers (
        number TEXT NOT NULL UNIQUE,
        match_key TEXT NOT NULL
    )"#,
    r#"CREATE TABLE IF NOT EXISTS speed_dials (
        dial_key TEXT NOT NULL UNIQUE, 
        tel TEXT NOT NULL, 
        contact_id TEXT
    )"#,
    r#"CREATE TABLE IF NOT EXISTS groups (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE)"#,
    r#"CREATE TABLE IF NOT EXISTS group_contacts (
       id INTEGER PRIMARY KEY ASC, 
       group_id TEXT NOT NULL, 
       contact_id TEXT NOT NULL,
       FOREIGN KEY(group_id) REFERENCES groups(id) ON DELETE CASCADE,
       FOREIGN KEY(contact_id) REFERENCES contact_main(contact_id) ON DELETE CASCADE
    )"#,
    r#"CREATE TABLE IF NOT EXISTS sim_contact_hash (
        id TEXT PRIMARY KEY,
        hash TEXT,
        FOREIGN KEY(id) REFERENCES contact_main(contact_id) ON DELETE CASCADE
    )"#,
];

impl DatabaseUpgrader for ContactsSchemaManager {
    fn upgrade(&mut self, from: u32, to: u32, connection: &mut Connection) -> bool {
        // We only support version 1 currently.
        if !(from == 0 && to == 1) {
            return false;
        }

        for cmd in &UPGRADE_0_1_SQL {
            if let Err(err) = connection.execute(cmd, NO_PARAMS) {
                error!("Upgrade step failure: {}", err);
                return false;
            }
        }

        if let Err(err) = load_contacts_to_db(CONTACTS_PRELOAD_FILE_PATH, connection) {
            error!(
                "Failed to load default contacts from {}: {}",
                CONTACTS_PRELOAD_FILE_PATH, err
            );
        } else {
            debug!(
                "Default contacts loaded successfully from {}",
                CONTACTS_PRELOAD_FILE_PATH
            );
        }

        true
    }
}

// This function creates a concatenated form of the phone number,
// to ensure that we don't consider equals numbers that have
// the same national representation but different international representations.
fn format_phone_number(number: &str) -> String {
    let countr_code = match AndroidProperties::get("persist.device.countrycode", "US") {
        Ok(value) => match Id::from_str(&value) {
            Ok(code) => code,
            Err(_) => Id::US,
        },
        Err(_) => Id::US,
    };

    if let Ok(phone_number) = phonenumber::parse(Some(countr_code), &number) {
        let mut result = String::new();
        result.push_str(&phone_number.format().mode(Mode::International).to_string());
        result.push_str(&phone_number.format().mode(Mode::National).to_string());
        result.push_str(&phone_number.format().mode(Mode::Rfc3966).to_string());
        result.push_str(&phone_number.format().mode(Mode::E164).to_string());
        result
    } else {
        // Parse error,use origin number.
        number.to_string()
    }
}

pub fn row_to_contact_id(row: &Row) -> Result<String, Error> {
    let column = row.column_index("contact_id")?;
    Ok(row.get(column)?)
}

impl From<SortOption> for String {
    fn from(value: SortOption) -> String {
        match value {
            SortOption::GivenName => "given_name".to_string(),
            SortOption::FamilyName => "family_name".to_string(),
            SortOption::Name => "name".to_string(),
        }
    }
}

impl From<Order> for String {
    fn from(value: Order) -> String {
        match value {
            Order::Ascending => "ASC".to_string(),
            Order::Descending => "DESC".to_string(),
        }
    }
}

#[derive(Debug)]
struct MainRowData {
    contact_id: String,
    name: String,
    family_name: String,
    given_name: String,
    tel_json: String,
    email_json: String,
    photo_type: String,
    photo_blob: Vec<u8>,
    published: i64,
    updated: i64,
    bday: i64,
    anniversary: i64,
    category: String,
    category_json: String,
}

#[derive(Debug)]
struct AdditionalRowData {
    contact_id: String,
    data_type: String,
    value: String,
}

impl Default for ContactInfo {
    fn default() -> Self {
        ContactInfo {
            id: None,
            published: Some(SystemTime::from(UNIX_EPOCH)),
            updated: Some(SystemTime::from(UNIX_EPOCH)),
            bday: Some(SystemTime::from(UNIX_EPOCH)),
            anniversary: Some(SystemTime::from(UNIX_EPOCH)),
            sex: None,
            gender_identity: None,
            ringtone: None,
            photo_type: None,
            photo_blob: None,
            addresses: None,
            email: None,
            url: None,
            name: None,
            tel: None,
            honorific_prefix: None,
            given_name: None,
            phonetic_given_name: None,
            additional_name: None,
            family_name: None,
            phonetic_family_name: None,
            honorific_suffix: None,
            nickname: None,
            category: None,
            org: None,
            job_title: None,
            note: None,
            groups: None,
            ice_position: 0,
        }
    }
}

struct SimContactHash {
    key: String,
    value: String,
}

impl Hash for SimContactInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.tel.hash(state);
        self.name.hash(state);
        self.email.hash(state);
        self.category.hash(state);
    }
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl From<&ContactInfo> for SimContactInfo {
    fn from(contact_info: &ContactInfo) -> Self {
        let tel_string = match &contact_info.tel {
            Some(tel) => {
                let tels: Vec<String> = tel.iter().map(|x| x.value.clone()).collect();
                tels.join("\u{001E}")
            }
            None => String::new(),
        };

        let email_string = match &contact_info.email {
            Some(email) => {
                let emails: Vec<String> = email.iter().map(|x| x.value.clone()).collect();
                emails.join("\u{001E}")
            }
            None => String::new(),
        };

        let category_string = match &contact_info.category {
            Some(category) => category.join("\u{001E}"),
            None => String::new(),
        };

        SimContactInfo {
            id: contact_info.id.clone().unwrap_or_default(),
            name: contact_info.name.clone().unwrap_or_default(),
            email: email_string,
            tel: tel_string,
            category: category_string,
        }
    }
}

impl From<&SimContactInfo> for ContactInfo {
    fn from(sim_contact_info: &SimContactInfo) -> Self {
        let mut contact = ContactInfo {
            id: Some(sim_contact_info.id.to_string()),
            name: Some(sim_contact_info.name.to_string()),
            family_name: Some(sim_contact_info.name.to_string()),
            given_name: Some(sim_contact_info.name.to_string()),
            ..Default::default()
        };

        let sim_tels: Vec<&str> = sim_contact_info.tel.split('\u{001E}').collect();
        let tels = sim_tels
            .iter()
            .map(|x| ContactTelField {
                atype: None,
                value: (*x).to_string(),
                pref: Some(false),
                carrier: None,
            })
            .collect();
        contact.tel = Some(tels);

        let sim_emails: Vec<&str> = sim_contact_info.email.split('\u{001E}').collect();
        let emails = sim_emails
            .iter()
            .map(|x| ContactField {
                atype: None,
                value: (*x).to_string(),
                pref: Some(false),
            })
            .collect();
        contact.email = Some(emails);

        let categories = sim_contact_info
            .category
            .split('\u{001E}')
            .map(|x| (*x).to_string())
            .collect();
        contact.category = Some(categories);

        contact.published = Some(SystemTime::from(std::time::SystemTime::now()));
        contact.updated = Some(SystemTime::from(std::time::SystemTime::now()));

        contact
    }
}

fn fill_vec_field<T>(field: &mut Option<Vec<T>>, value: T) {
    if let Some(fields) = field.as_mut() {
        fields.push(value);
    } else {
        *field = Some(vec![value]);
    }
}

fn save_vec_field(
    stmt: &mut Statement,
    id: &Option<String>,
    stype: &str,
    datas: &Option<Vec<String>>,
) -> Result<(), Error> {
    if let Some(values) = datas {
        for value in values {
            stmt.insert(&[&id as &dyn rusqlite::ToSql, &stype.to_string(), &value])?;
        }
    }
    Ok(())
}

fn save_str_field(
    stmt: &mut Statement,
    id: &Option<String>,
    stype: &str,
    data: &Option<String>,
) -> Result<(), Error> {
    if let Some(data) = data {
        stmt.insert(&[&id as &dyn rusqlite::ToSql, &stype.to_string(), &data])?;
    }
    Ok(())
}

// Converts a value to Some(val) if it's not the default for this type,
// and to None otherwise.
fn maybe<T>(val: T) -> Option<T>
where
    T: Default + PartialEq,
{
    if val == T::default() {
        None
    } else {
        Some(val)
    }
}

impl ContactInfo {
    pub fn fill_main_data(&mut self, id: &str, conn: &Connection) -> Result<(), Error> {
        self.id = Some(id.into());
        let mut stmt = conn.prepare(
            "SELECT contact_id, name, family_name, given_name, tel_json, email_json,
        photo_type, photo_blob, published, updated, bday, anniversary, category, category_json FROM
        contact_main WHERE contact_id=:id",
        )?;

        let rows =
            stmt.query_map_named(&[(":id", &(self.id.clone().unwrap_or_default()))], |row| {
                Ok(MainRowData {
                    contact_id: row.get(0)?,
                    name: row.get(1)?,
                    family_name: row.get(2)?,
                    given_name: row.get(3)?,
                    tel_json: row.get(4)?,
                    email_json: row.get(5)?,
                    photo_type: row.get(6)?,
                    photo_blob: row.get(7)?,
                    published: row.get(8)?,
                    updated: row.get(9)?,
                    bday: row.get(10)?,
                    anniversary: row.get(11)?,
                    category: row.get(12)?,
                    category_json: row.get(13)?,
                })
            })?;

        let mut rows = rows.peekable();
        if rows.peek().is_none() {
            return Err(Error::InvalidContactId(
                "Try to fill contact with invalid contact id".to_string(),
            ));
        }

        for result_row in rows {
            let row = result_row?;
            debug!("Current row data is {:#?}", row);
            self.name = maybe(row.name);
            self.family_name = maybe(row.family_name);
            self.given_name = maybe(row.given_name);

            if !row.tel_json.is_empty() {
                let tel: Vec<ContactTelField> = serde_json::from_str(&row.tel_json)?;
                self.tel = Some(tel);
            }

            if !row.email_json.is_empty() {
                let email: Vec<ContactField> = serde_json::from_str(&row.email_json)?;
                self.email = Some(email);
            }

            if !row.category_json.is_empty() {
                let category: Vec<String> = serde_json::from_str(&row.category_json)?;
                self.category = Some(category);
            }

            self.photo_type = maybe(row.photo_type);
            self.photo_blob = maybe(row.photo_blob);

            if let Some(time) = UNIX_EPOCH.checked_add(Duration::from_secs(row.published as u64)) {
                self.published = Some(SystemTime::from(time));
            }

            if let Some(time) = UNIX_EPOCH.checked_add(Duration::from_secs(row.updated as u64)) {
                self.updated = Some(SystemTime::from(time));
            }

            if let Some(time) = UNIX_EPOCH.checked_add(Duration::from_secs(row.bday as u64)) {
                self.bday = Some(SystemTime::from(time));
            }

            if let Some(time) = UNIX_EPOCH.checked_add(Duration::from_secs(row.anniversary as u64))
            {
                self.anniversary = Some(SystemTime::from(time));
            }
        }
        Ok(())
    }

    pub fn fill_additional_data(&mut self, id: &str, conn: &Connection) -> Result<(), Error> {
        self.id = Some(id.into());
        let mut stmt = conn.prepare(
            "SELECT contact_id, data_type, value FROM contact_additional WHERE contact_id=:id",
        )?;
        let rows = stmt.query_map_named(&[(":id", &id)], |row| {
            Ok(AdditionalRowData {
                contact_id: row.get(0)?,
                data_type: row.get(1)?,
                value: row.get(2)?,
            })
        })?;

        for result_row in rows {
            let row = result_row?;
            if row.data_type == "honorific_prefix" {
                fill_vec_field(&mut self.honorific_prefix, row.value);
            } else if row.data_type == "phonetic_given_name" {
                self.phonetic_given_name = maybe(row.value);
            } else if row.data_type == "phonetic_family_name" {
                self.phonetic_family_name = maybe(row.value);
            } else if row.data_type == "additional_name" {
                fill_vec_field(&mut self.additional_name, row.value);
            } else if row.data_type == "honorific_suffix" {
                fill_vec_field(&mut self.honorific_suffix, row.value);
            } else if row.data_type == "nickname" {
                fill_vec_field(&mut self.nickname, row.value);
            } else if row.data_type == "org" {
                fill_vec_field(&mut self.org, row.value);
            } else if row.data_type == "job_title" {
                fill_vec_field(&mut self.job_title, row.value);
            } else if row.data_type == "note" {
                fill_vec_field(&mut self.note, row.value);
            } else if row.data_type == "addresses" {
                if !&row.value.is_empty() {
                    let addr: Vec<Address> = serde_json::from_str(&row.value)?;
                    self.addresses = Some(addr);
                }
            } else if row.data_type == "ringtone" {
                self.ringtone = maybe(row.value);
            } else if row.data_type == "gender_identity" {
                self.gender_identity = maybe(row.value);
            } else if row.data_type == "sex" {
                self.sex = maybe(row.value);
            } else if row.data_type == "url" {
                if !row.value.is_empty() {
                    let url: Vec<ContactField> = serde_json::from_str(&row.value)?;
                    self.url = Some(url);
                }
            } else if row.data_type == "groups" {
                fill_vec_field(&mut self.groups, row.value);
            } else if row.data_type == "ice_position" {
                self.ice_position = row.value.parse().unwrap_or(0);
            } else {
                error!("Unknown type in additional data: {}", row.data_type);
            }
        }
        Ok(())
    }

    pub(crate) fn save_main_data(&self, tx: &rusqlite::Transaction) -> Result<(), Error> {
        let mut stmt_ins = tx.prepare("INSERT INTO contact_main (contact_id, name, family_name, given_name, 
            tel_number, tel_json, email, email_json, photo_type, photo_blob, published, updated, bday, 
            anniversary, category, category_json) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")?;

        let mut tel_number: String = "\u{001E}".to_string();
        let mut tel_json = String::new();
        if let Some(tels) = &self.tel {
            for tel in tels {
                tel_number += &tel.value;
                // Seperate with unprintable character, used for find.
                tel_number.push('\u{001E}');

                // Store the International, National, Rfc3966, E164 format of number used for search.
                tel_number.push_str(&format_phone_number(&tel.value));
                tel_number.push('\u{001E}');
            }
            // The tel_json is used for restore the tel struct.
            tel_json = serde_json::to_string(tels)?;
        }

        let mut email_address = "\u{001E}".to_string();
        let mut email_json = String::new();
        if let Some(emails) = &self.email {
            for email in emails {
                email_address += &email.value;
                // Seperate with unprintable character, used for find.
                email_address.push('\u{001E}');
            }
            // The email_json is used for restore the email struct.
            email_json = serde_json::to_string(emails)?;
        }

        let mut category = "\u{001E}".to_string();
        let mut category_json = String::new();
        if let Some(categories) = &self.category {
            for item in categories {
                category += item;
                // Seperate with unprintable character, used for find.
                category.push('\u{001E}');
            }
            // The category_json is used for restore the category array.
            category_json = serde_json::to_string(categories)?;
        }

        let epoch: common::SystemTime = UNIX_EPOCH.into();

        let mut published = 0;
        if let Ok(duration) = self
            .published
            .as_ref()
            .unwrap_or(&epoch)
            .duration_since(UNIX_EPOCH)
        {
            published = duration.as_secs() as i64;
        }

        let mut updated = 0;
        if let Ok(duration) = self
            .updated
            .as_ref()
            .unwrap_or(&epoch)
            .duration_since(UNIX_EPOCH)
        {
            updated = duration.as_secs() as i64;
        }

        let mut bday = 0;
        if let Ok(duration) = self
            .bday
            .as_ref()
            .unwrap_or(&epoch)
            .duration_since(UNIX_EPOCH)
        {
            bday = duration.as_secs() as i64;
        }

        let mut anniversary = 0;
        if let Ok(duration) = self
            .anniversary
            .as_ref()
            .unwrap_or(&epoch)
            .duration_since(UNIX_EPOCH)
        {
            anniversary = duration.as_secs() as i64;
        }

        stmt_ins.insert(&[
            &self.id as &dyn rusqlite::ToSql,
            &self.name.clone().unwrap_or_default(),
            &self.family_name.clone().unwrap_or_default(),
            &self.given_name.clone().unwrap_or_default(),
            &tel_number,
            &tel_json,
            &email_address,
            &email_json,
            &self.photo_type.clone().unwrap_or_default(),
            &self.photo_blob.clone().unwrap_or_default(),
            &published,
            &updated,
            &bday,
            &anniversary,
            &category,
            &category_json,
        ])?;

        // Update sim_contact_hash if it is a sim contact.
        // Update after insert contact_main due to foreign key constraint.
        if let Some(categories) = &self.category {
            if categories.contains(&"SIM".to_string()) {
                // Add or Update sim contact all need to update hash.
                // So delete first and then insert.
                tx.execute("DELETE FROM sim_contact_hash WHERE id = ?", &[&self.id])?;
                let sim_info: SimContactInfo = self.into();
                let hash = Some(hash(&sim_info).to_string());
                tx.execute(
                    "INSERT INTO sim_contact_hash (id, hash) VALUES(?, ?)",
                    &[&self.id, &hash],
                )?;
            }
        }

        Ok(())
    }

    pub(crate) fn save_additional_data(&self, tx: &rusqlite::Transaction) -> Result<(), Error> {
        let mut stmt = tx.prepare(
            "INSERT INTO contact_additional (contact_id, data_type, value) VALUES(?, ?, ?)",
        )?;
        save_vec_field(
            &mut stmt,
            &self.id,
            "honorific_prefix",
            &self.honorific_prefix,
        )?;
        save_vec_field(
            &mut stmt,
            &self.id,
            "additional_name",
            &self.additional_name,
        )?;
        save_vec_field(
            &mut stmt,
            &self.id,
            "honorific_suffix",
            &self.honorific_suffix,
        )?;
        save_vec_field(&mut stmt, &self.id, "nickname", &self.nickname)?;
        save_vec_field(&mut stmt, &self.id, "org", &self.org)?;
        save_vec_field(&mut stmt, &self.id, "job_title", &self.job_title)?;
        save_vec_field(&mut stmt, &self.id, "note", &self.note)?;
        save_str_field(&mut stmt, &self.id, "sex", &self.sex)?;
        save_str_field(
            &mut stmt,
            &self.id,
            "gender_identity",
            &self.gender_identity,
        )?;
        save_str_field(&mut stmt, &self.id, "ringtone", &self.ringtone)?;
        save_str_field(
            &mut stmt,
            &self.id,
            &"phonetic_given_name",
            &self.phonetic_given_name,
        )?;
        save_str_field(
            &mut stmt,
            &self.id,
            &"phonetic_family_name",
            &self.phonetic_family_name,
        )?;

        if self.ice_position != 0 {
            save_str_field(
                &mut stmt,
                &self.id,
                "ice_position",
                &Some(self.ice_position.to_string()),
            )?;
        }

        if let Some(addresses) = &self.addresses {
            let json = serde_json::to_string(addresses)?;
            save_str_field(&mut stmt, &self.id, "addresses", &Some(json))?;
        }

        if let Some(url) = &self.url {
            let json = serde_json::to_string(url)?;
            save_str_field(&mut stmt, &self.id, "url", &Some(json))?;
        }

        if let Some(groups) = &self.groups {
            for group in groups {
                stmt.insert(&[&self.id, &Some("groups".into()), &Some(group.clone())])?;
                // Update the group_contacts when contact with group info.
                let mut stmt_group =
                    tx.prepare("INSERT INTO group_contacts (group_id, contact_id) VALUES(?, ?)")?;
                stmt_group.insert(&[&group, &self.id.clone().unwrap_or_default()])?;
            }
        }

        Ok(())
    }
}

// Creates a contacts database with a proper updater.
// SQlite manages itself thread safety so we can use
// multiple ones without having to use Rust mutexes.
pub fn create_db() -> SqliteDb {
    let db = match SqliteDb::open(DB_PATH, &mut ContactsSchemaManager {}, 1) {
        Ok(db) => db,
        Err(err) => panic!("Failed to open contacts db: {}", err),
    };
    if let Err(err) = db.enable_wal() {
        error!("Failed to enable WAL mode on contacts db: {}", err);
    }

    db
}

pub struct ContactsDb {
    // The underlying sqlite db.
    db: SqliteDb,
    // Handle to the event broadcaster to fire events when changing contacts.
    event_broadcaster: ContactsFactoryEventBroadcaster,
    // Thread pool used for cursors.
    cursors: ThreadPool,
}

impl ContactsDb {
    pub fn new(event_broadcaster: ContactsFactoryEventBroadcaster) -> Self {
        Self {
            db: create_db(),
            event_broadcaster,
            cursors: ThreadPool::with_name("ContactsDbCursor".into(), 5),
        }
    }

    pub fn add_dispatcher(&mut self, dispatcher: &ContactsFactoryEventDispatcher) -> DispatcherId {
        self.event_broadcaster.add(dispatcher)
    }

    pub fn remove_dispatcher(&mut self, id: DispatcherId) {
        self.event_broadcaster.remove(id)
    }

    pub fn clear_contacts(&mut self) -> Result<(), Error> {
        debug!("ContactsDb::clear_contacts");
        let conn = self.db.mut_connection();
        let contact_ids = {
            let mut stmt = conn.prepare("SELECT contact_id FROM contact_main")?;
            let rows = stmt.query_map(NO_PARAMS, |row| Ok(row_to_contact_id(row)))?;
            rows_to_vec(rows)
        };

        let tx = conn.transaction()?;
        tx.execute("UPDATE speed_dials SET contact_id = ''", NO_PARAMS)?;
        tx.execute("DELETE FROM contact_main", NO_PARAMS)?;
        tx.commit()?;

        if !contact_ids.is_empty() {
            let contacts: Vec<ContactInfo> = contact_ids
                .iter()
                .map(|x| ContactInfo {
                    id: match x {
                        Ok(id) => Some(id.to_string()),
                        Err(_) => None,
                    },
                    ..Default::default()
                })
                .collect();
            let event = ContactsChangeEvent {
                reason: ChangeReason::Remove,
                contacts: Some(contacts),
            };
            self.event_broadcaster.broadcast_contacts_change(event);
        }

        Ok(())
    }

    pub fn remove(&mut self, contact_ids: &[String]) -> Result<(), Error> {
        debug!("ContactsDb::remove contacts");
        let connection = self.db.mut_connection();
        let count: i32 = {
            let mut sql = String::from("SELECT COUNT(*) FROM contact_main WHERE contact_id in (");
            for _i in 1..contact_ids.len() {
                sql += "?,";
            }
            sql += "?)";

            debug!("verify has none exist id in remove sql is: {}", sql);

            let mut stmt = connection.prepare(&sql)?;

            stmt.query_row(contact_ids, |r| Ok(r.get_unwrap(0)))?
        };

        if count != contact_ids.len() as i32 {
            return Err(Error::InvalidContactId(
                "Try to remove none exist contact".to_string(),
            ));
        }

        let mut contacts = vec![];
        let tx = connection.transaction()?;
        {
            for contact_id in contact_ids {
                tx.execute(
                    "UPDATE speed_dials SET contact_id = '' WHERE contact_id = ?",
                    &[&contact_id],
                )?;
                tx.execute(
                    "DELETE FROM contact_main WHERE contact_id = ?",
                    &[&contact_id],
                )?;
                let contact = ContactInfo {
                    id: Some(contact_id.to_string()),
                    ..Default::default()
                };
                contacts.push(contact);
            }
        }
        tx.commit()?;
        let event = ContactsChangeEvent {
            reason: ChangeReason::Remove,
            contacts: Some(contacts),
        };

        self.event_broadcaster.broadcast_contacts_change(event);
        Ok(())
    }

    pub fn save(&mut self, contacts: &[ContactInfo], is_update: bool) -> Result<(), Error> {
        debug!("ContactsDb::add {} contacts", contacts.len());
        let connection = self.db.mut_connection();
        let mut new_contacts = vec![];
        debug!("ContactsDb::save is_update:{}", is_update);
        let tx = connection.transaction()?;
        {
            for contact_info in contacts {
                let mut contact = contact_info.clone();
                if is_update {
                    if contact.id.is_none() {
                        debug!("Try to update a contact without contact id, ignore it");
                        continue;
                    }
                    contact.updated = Some(SystemTime::from(std::time::SystemTime::now()));
                    tx.execute("DELETE FROM sim_contact_hash WHERE id = ?", &[&contact.id])?;
                    tx.execute(
                        "DELETE FROM contact_additional WHERE contact_id = ?",
                        &[&contact.id],
                    )?;
                    tx.execute(
                        "DELETE FROM group_contacts WHERE contact_id = ?",
                        &[&contact.id],
                    )?;
                    tx.execute(
                        "DELETE FROM contact_main WHERE contact_id = ?",
                        &[&contact.id],
                    )?;
                    contact.updated = Some(SystemTime::from(std::time::SystemTime::now()));
                } else {
                    contact.id = Some(Uuid::new_v4().to_string());
                    contact.published = Some(SystemTime::from(std::time::SystemTime::now()));
                }
                debug!("Save current contact id is {:?}", contact.id);

                if let Err(err) = contact.save_main_data(&tx) {
                    error!("save_main_data error: {}, continue", err);
                    continue;
                }
                if let Err(err) = contact.save_additional_data(&tx) {
                    error!("save_additional_data error: {}, continue", err);
                    continue;
                }
                new_contacts.push(contact);
            }
        }
        tx.commit()?;

        let event = ContactsChangeEvent {
            reason: if is_update {
                ChangeReason::Update
            } else {
                ChangeReason::Create
            },
            contacts: Some(new_contacts),
        };
        self.event_broadcaster.broadcast_contacts_change(event);
        Ok(())
    }

    pub fn get(&self, id: &str, only_main_data: bool) -> Result<ContactInfo, Error> {
        debug!(
            "ContactsDb::get id {}, only_main_data {}",
            id, only_main_data
        );
        let mut contact = ContactInfo::default();
        let conn = self.db.connection();
        contact.fill_main_data(&id, &conn)?;
        if !only_main_data {
            contact.fill_additional_data(&id, &conn)?;
        }
        Ok(contact)
    }

    pub fn count(&self) -> Result<u32, Error> {
        debug!("ContactsDb::count");
        let mut stmt = self
            .db
            .connection()
            .prepare("SELECT COUNT(contact_id) FROM contact_main")?;

        let count = stmt.query_row(NO_PARAMS, |r| Ok(r.get_unwrap(0)))?;

        Ok(count)
    }

    pub fn get_all(
        &self,
        options: ContactSortOptions,
        batch_size: i64,
        only_main_data: bool,
    ) -> Option<ContactDbCursor> {
        debug!(
            "ContactsDb::get_all options {:#?}, batch_size {}, only_main_data {}",
            options, batch_size, only_main_data
        );

        Some(ContactDbCursor::new(
            batch_size,
            only_main_data,
            &self.cursors,
            move |connection| {
                let field: String = options.sort_by.into();
                let order: String = options.sort_order.into();
                debug!("field = {}", field);
                debug!("order = {}", order);

                let order: String = options.sort_order.into();
                let order_field = match options.sort_by {
                    SortOption::FamilyName => format!(
                        "family_name COLLATE NOCASE {}, given_name COLLATE NOCASE {}",
                        order, order
                    ),
                    SortOption::GivenName => format!(
                        "given_name COLLATE NOCASE {}, family_name COLLATE NOCASE {}",
                        order, order
                    ),
                    SortOption::Name => format!("name COLLATE NOCASE {}", order),
                };

                let sql = format!(
                    "SELECT contact_id FROM contact_main ORDER BY {}",
                    order_field
                );
                debug!("get_all sql is {}", sql);
                let statement = match connection.prepare(&sql) {
                    Ok(statement) => statement,
                    Err(err) => {
                        error!("Failed to prepare `get_all` statement `{}`: {}", sql, err);
                        return None;
                    }
                };
                Some(statement)
            },
        ))
    }

    pub fn find(
        &self,
        options: ContactFindSortOptions,
        batch_size: i64,
    ) -> Option<ContactDbCursor> {
        debug!("ContactsDb::find {:#?}, batch_size {}", options, batch_size);
        Some(ContactDbCursor::new(
            batch_size,
            options.only_main_data,
            &self.cursors,
            move |connection| {
                let mut sql = String::from("SELECT contact_id FROM contact_main WHERE ");
                let mut params = vec![];
                for (n, filter_by) in options.filter_by.iter().enumerate() {
                    if n != 0 {
                        sql.push_str(" OR ");
                    }
                    match *filter_by {
                        FilterByOption::Name => {
                            sql.push_str("name LIKE ?");
                        }
                        FilterByOption::GivenName => {
                            sql.push_str("given_name LIKE ?");
                        }
                        FilterByOption::FamilyName => {
                            sql.push_str("family_name LIKE ?");
                        }
                        FilterByOption::Email => {
                            sql.push_str("email LIKE ?");
                        }
                        FilterByOption::Tel => {
                            // Search contact tel by contains should use tel_json field.
                            // The tel_number field contains the national code, search a
                            // number which is nationl code may return wrong result.
                            if options.filter_option == FilterOption::Contains {
                                sql.push_str("tel_json LIKE ?");
                            } else {
                                sql.push_str("tel_number LIKE ?");
                            }
                        }
                        FilterByOption::Category => {
                            sql.push_str("category LIKE ?");
                        }
                    }

                    let value = match options.filter_option {
                        FilterOption::StartsWith => {
                            if *filter_by == FilterByOption::Email
                                || *filter_by == FilterByOption::Tel
                            {
                                // The tel_number and email will store like:"\u{001E}88888\u{001E}99999\u{001E}.
                                // StartsWith means contain %\u{001E}{}%.
                                format!("%\u{001E}{}%", options.filter_value)
                            } else {
                                format!("{}%", options.filter_value)
                            }
                        }
                        FilterOption::FuzzyMatch => {
                            // Only used for tel
                            // Matching from back to front, If the filter_value length is greater
                            // Than MIN_MATCH_DIGITS, take the last MIN_MATCH_DIGITS length string.
                            let filter_value = &options.filter_value;
                            let mut slice = String::new();
                            if filter_value.len() > MIN_MATCH_DIGITS {
                                let start = filter_value.len() - MIN_MATCH_DIGITS;
                                if let Some(value_slice) =
                                    filter_value.get(start..filter_value.len())
                                {
                                    slice = value_slice.to_string()
                                }
                                format!("%{}\u{001E}%", slice)
                            } else {
                                format!("%{}\u{001E}%", filter_value)
                            }
                        }
                        FilterOption::Contains => format!("%{}%", options.filter_value),
                        FilterOption::Equals => {
                            if *filter_by == FilterByOption::Email
                                || *filter_by == FilterByOption::Tel
                            {
                                // The email and tel number will store like:"\u{001E}88888\u{001E}99999\u{001E}".
                                // For equal it means to contains "%\u{001E}{}\u{001E}%".
                                format!("%\u{001E}{}\u{001E}%", options.filter_value)
                            } else {
                                options.filter_value.to_string()
                            }
                        }
                        FilterOption::Match => {
                            // Match only used for Tel.
                            if *filter_by == FilterByOption::Tel {
                                // The tel number will store like:"\u{001E}88888\u{001E}99999\u{001E}".
                                // Convert to the International, National, Rfc3966, E164 format.
                                format!(
                                    "%\u{001E}{}\u{001E}%",
                                    format_phone_number(&options.filter_value)
                                )
                            } else {
                                String::new()
                            }
                        }
                    };

                    debug!("find filter value is {}", value);
                    params.push(value);
                }

                let order: String = options.sort_order.into();
                let order_filed = match options.sort_by {
                    SortOption::FamilyName => format!(
                        "family_name COLLATE NOCASE {}, given_name COLLATE NOCASE {}",
                        order, order
                    ),
                    SortOption::GivenName => format!(
                        "given_name COLLATE NOCASE {}, family_name COLLATE NOCASE {}",
                        order, order
                    ),
                    SortOption::Name => format!("name COLLATE NOCASE {}", order),
                };

                if !order_filed.is_empty() {
                    sql.push_str(" ORDER BY ");
                    sql.push_str(&order_filed);
                }

                debug!("find sql is {}", sql);

                let mut statement = match connection.prepare(&sql) {
                    Ok(statement) => statement,
                    Err(err) => {
                        error!("Failed to prepare `find` statement: {} error: {}", sql, err);
                        return None;
                    }
                };

                for (n, param) in params.iter().enumerate() {
                    debug!("current n is {}, param = {}", n, params[n]);
                    // SQLite binding indexes are 1 based, not 0 based...
                    if let Err(err) = statement.raw_bind_parameter(n + 1, param) {
                        error!(
                            "Failed to bind #{} `find` parameter to `{}`: {}",
                            n, sql, err
                        );
                        return None;
                    }
                }

                Some(statement)
            },
        ))
    }

    pub fn set_ice(&mut self, contact_id: &str, position: i64) -> Result<(), Error> {
        let conn = self.db.connection();
        let contact_id_count: i32 = {
            let sql = String::from("SELECT COUNT(*) FROM contact_main WHERE contact_id = ?");
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_row(&[&contact_id], |r| Ok(r.get_unwrap(0)))?
        };

        if contact_id_count != 1 {
            return Err(Error::InvalidContactId(
                "Try to set_ice with invalid contact id".to_string(),
            ));
        }

        let position_count: i32 = {
            let sql = String::from(
                "SELECT COUNT(*) FROM contact_additional WHERE data_type = 'ice_position' AND value = ?",
            );
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_row(&[&position], |r| Ok(r.get_unwrap(0)))?
        };

        if position_count != 0 {
            return Err(Error::IcePositionUsed(
                "Try to set_ice with position already used".to_string(),
            ));
        }

        let item_count: i32 = {
            let sql = String::from(
                "SELECT COUNT(*) FROM contact_additional WHERE data_type = 'ice_position' AND contact_id = ?",
            );
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_row(&[&contact_id], |r| Ok(r.get_unwrap(0)))?
        };

        if item_count != 0 {
            conn.execute_named(
                "UPDATE contact_additional SET value = :position WHERE contact_id = :contact_id
                AND data_type = 'ice_position'",
                &[(":position", &position), (":contact_id", &contact_id)],
            )?;
        } else {
            conn.execute_named(
                "INSERT INTO contact_additional (contact_id, data_type, value) 
                VALUES (:contact_id, 'ice_position', :position)",
                &[(":contact_id", &contact_id), (":position", &position)],
            )?;
        }

        Ok(())
    }

    pub fn remove_ice(&mut self, contact_id: &str) -> Result<(), Error> {
        let conn = self.db.connection();
        let count: i32 = {
            let sql = String::from("SELECT COUNT(*) FROM contact_main WHERE contact_id = ?");
            let mut stmt = conn.prepare(&sql)?;
            stmt.query_row(&[&contact_id], |r| Ok(r.get_unwrap(0)))?
        };

        if count != 1 {
            return Err(Error::InvalidContactId(
                "Try to remove_ice with invalid contact id".to_string(),
            ));
        }

        conn.execute(
            "DELETE FROM contact_additional WHERE contact_id = ? AND data_type = 'ice_position'",
            &[contact_id],
        )?;

        Ok(())
    }

    pub fn get_all_ice(&mut self) -> Result<Vec<IceInfo>, Error> {
        debug!("ContactsDb::get_all_ice");
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT value, contact_id FROM contact_additional WHERE
            data_type = 'ice_position' AND value != '0' ORDER BY value ASC",
        )?;

        let rows = stmt.query_map(NO_PARAMS, |row| {
            Ok(IceInfo {
                position: {
                    let value: String = row.get(0)?;
                    value.parse().unwrap_or(0)
                },
                contact_id: row.get(1)?,
            })
        })?;

        Ok(rows_to_vec(rows))
    }

    pub fn import_vcf(&mut self, vcf: &str) -> Result<usize, Error> {
        debug!("import_vcf {}", vcf.len());
        let parser = ical::VcardParser::new(BufReader::new(vcf.as_bytes()));
        let mut contacts = vec![];
        for item in parser {
            if let Ok(vcard) = item {
                // Initialize the contact with default values.
                let mut contact = ContactInfo::default();
                for prop in vcard.properties {
                    if prop.name == "EMAIL" {
                        if let Some(email_vcard) = &prop.value {
                            fill_vec_field(
                                &mut contact.email,
                                ContactField {
                                    atype: None,
                                    value: email_vcard.clone(),
                                    pref: Some(false),
                                },
                            );
                        }
                    } else if prop.name == "TEL" {
                        if let Some(tel_vcard) = &prop.value {
                            fill_vec_field(
                                &mut contact.tel,
                                ContactTelField {
                                    atype: None,
                                    value: tel_vcard.clone(),
                                    pref: Some(false),
                                    carrier: None,
                                },
                            );
                        }
                    } else if prop.name == "FN" {
                        contact.name = prop.value;
                    } else if prop.name == "TITLE" {
                        fill_vec_field(
                            &mut contact.job_title,
                            prop.value.unwrap_or_else(|| "".into()),
                        );
                    }
                }
                debug!("contact in vcard is : {:?}", contact);
                contacts.push(contact);
            }
        }
        self.save(&contacts, false).map(|_| contacts.len())
    }

    pub fn add_blocked_number(&mut self, number: &str) -> Result<(), Error> {
        debug!("ContactsDb::add_blocked_number number:{}", number);

        let conn = self.db.connection();
        let match_key = format_phone_number(number);
        let mut stmt =
            conn.prepare("INSERT INTO blocked_numbers (number, match_key) VALUES (?, ?)")?;
        let size = stmt.execute(&[number, &match_key])?;
        if size > 0 {
            let event = BlockedNumberChangeEvent {
                reason: ChangeReason::Create,
                number: number.to_string(),
            };
            self.event_broadcaster.broadcast_blockednumber_change(event);
        }
        debug!("ContactsDb::add_blocked_number OK {}", size);
        Ok(())
    }

    pub fn remove_blocked_number(&mut self, number: &str) -> Result<(), Error> {
        debug!("ContactsDb::remove_blocked_number number:{}", number);

        let conn = self.db.connection();
        let mut stmt = conn.prepare("DELETE FROM blocked_numbers WHERE number = ?")?;
        let size = stmt.execute(&[number])?;
        if size > 0 {
            let event = BlockedNumberChangeEvent {
                reason: ChangeReason::Remove,
                number: number.to_string(),
            };
            self.event_broadcaster.broadcast_blockednumber_change(event);
        }
        debug!("ContactsDb::remove_blocked_number OK size:{}", size);
        Ok(())
    }

    pub fn get_all_blocked_numbers(&mut self) -> Result<Vec<String>, Error> {
        debug!("ContactsDb::get_all_blocked_numbers");
        let conn = self.db.connection();
        let mut stmt = conn.prepare("SELECT number FROM blocked_numbers")?;

        let rows = stmt.query_map(NO_PARAMS, |row| row.get(0))?;
        Ok(rows_to_vec(rows))
    }

    pub fn find_blocked_numbers(
        &mut self,
        options: BlockedNumberFindOptions,
    ) -> Result<Vec<String>, Error> {
        debug!("ContactsDb::find_blocked_numbers options:{:?}", &options);
        let conn = self.db.connection();
        let mut stmt;
        if options.filter_option == FilterOption::Match {
            stmt =
                conn.prepare("SELECT number FROM blocked_numbers WHERE match_key LIKE :param")?;
        } else {
            stmt = conn.prepare("SELECT number FROM blocked_numbers WHERE number LIKE :param")?;
        }

        let param = match options.filter_option {
            FilterOption::StartsWith => format!("{}%", options.filter_value),
            FilterOption::FuzzyMatch => {
                // Matching from back to front, If the filter_value length is greater
                // than MIN_MATCH_DIGITS, take the last MIN_MATCH_DIGITS length string.
                let mut filter_value = options.filter_value;
                if filter_value.len() > MIN_MATCH_DIGITS {
                    let start = filter_value.len() - MIN_MATCH_DIGITS;
                    if let Some(value_slice) = filter_value.get(start..filter_value.len()) {
                        filter_value = value_slice.to_string()
                    }
                }
                format!("%{}", filter_value)
            }
            FilterOption::Contains => format!("%{}%", options.filter_value),
            FilterOption::Equals => options.filter_value,
            FilterOption::Match => format_phone_number(&options.filter_value),
        };

        let rows = stmt.query_map_named(&[(":param", &param)], |row| row.get(0))?;
        Ok(rows_to_vec(rows))
    }

    pub fn get_speed_dials(&mut self) -> Result<Vec<SpeedDialInfo>, Error> {
        debug!("ContactsDb::get_speed_dials");
        let conn = self.db.connection();
        let mut stmt = conn.prepare("SELECT * FROM speed_dials")?;

        let rows = stmt.query_map(NO_PARAMS, |row| {
            Ok(SpeedDialInfo {
                dial_key: row.get(0)?,
                tel: row.get(1)?,
                contact_id: row.get(2)?,
            })
        })?;

        Ok(rows_to_vec(rows))
    }

    pub fn add_speed_dial(
        &mut self,
        dial_key: &str,
        tel: &str,
        contact_id: &str,
    ) -> Result<(), Error> {
        debug!(
            "ContactsDb::add_speed_dial, dial_key:{}, tel:{}, contact_id:{}",
            dial_key, tel, contact_id
        );
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("INSERT INTO speed_dials (dial_key, tel, contact_id) VALUES (?1, ?2, ?3)")?;
        let size = stmt.execute(&[dial_key, tel, contact_id])?;
        if size > 0 {
            let event = SpeedDialChangeEvent {
                reason: ChangeReason::Create,
                speeddial: SpeedDialInfo {
                    dial_key: dial_key.to_string(),
                    tel: tel.to_string(),
                    contact_id: contact_id.to_string(),
                },
            };
            debug!("ContactsDb::add_speed_dial event ={:?}", event);
            self.event_broadcaster.broadcast_speeddial_change(event);
        }
        debug!("ContactsDb::add_speed_dial Ok {}", size);
        Ok(())
    }

    pub fn update_speed_dial(
        &mut self,
        dial_key: &str,
        tel: &str,
        contact_id: &str,
    ) -> Result<(), Error> {
        debug!(
            "ContactsDb::update_speed_dial, dial_key:{}, tel:{}, contact_id:{}",
            dial_key, tel, contact_id
        );
        let conn = self.db.connection();
        let mut stmt =
            conn.prepare("UPDATE speed_dials SET tel = ?1, contact_id = ?2 WHERE dial_key = ?3")?;
        let size = stmt.execute(&[tel, contact_id, dial_key])?;
        if size > 0 {
            let event = SpeedDialChangeEvent {
                reason: ChangeReason::Update,
                speeddial: SpeedDialInfo {
                    dial_key: dial_key.to_string(),
                    tel: tel.to_string(),
                    contact_id: contact_id.to_string(),
                },
            };
            self.event_broadcaster.broadcast_speeddial_change(event);
        }
        debug!("ContactsDb::update_speed_dial Ok {}", size);
        Ok(())
    }

    pub fn remove_speed_dial(&mut self, dial_key: &str) -> Result<(), Error> {
        debug!("ContactsDb::remove_speed_dial");
        let conn = self.db.connection();
        let mut stmt = conn.prepare("DELETE FROM speed_dials WHERE dial_key = ?")?;
        let size = stmt.execute(&[dial_key])?;
        if size > 0 {
            let event = SpeedDialChangeEvent {
                reason: ChangeReason::Remove,
                speeddial: SpeedDialInfo {
                    dial_key: dial_key.to_string(),
                    tel: String::new(),
                    contact_id: String::new(),
                },
            };
            self.event_broadcaster.broadcast_speeddial_change(event);
        }
        debug!("ContactsDb::remove_speed_dial Ok {}", size);
        Ok(())
    }

    pub fn remove_group(&mut self, id: &str) -> Result<(), Error> {
        debug!("ContactsDb::remove_group id:{}", id);
        let connection = self.db.mut_connection();
        let tx = connection.transaction()?;
        tx.execute("DELETE FROM group_contacts WHERE group_id is ?", &[id])?;
        tx.execute(
            "DELETE FROM contact_additional WHERE data_type = 'groups' AND value IS ?",
            &[id],
        )?;
        tx.execute("DELETE FROM groups WHERE id is ?", &[id])?;
        tx.commit()?;
        let event = GroupChangeEvent {
            reason: ChangeReason::Remove,
            group: GroupInfo {
                name: String::new(),
                id: id.to_string(),
            },
        };
        self.event_broadcaster.broadcast_group_change(event);
        debug!("ContactsDb::remove_group OK ");
        Ok(())
    }

    pub fn add_group(&mut self, name: &str) -> Result<(), Error> {
        debug!("ContactsDb::add_group  name = {}", name);
        let id = Uuid::new_v4().to_string();
        let conn = self.db.connection();
        let size = conn.execute("INSERT INTO groups (id, name) VALUES(?, ?)", &[&id, name])?;
        if size > 0 {
            let event = GroupChangeEvent {
                reason: ChangeReason::Create,
                group: GroupInfo {
                    name: name.to_string(),
                    id,
                },
            };
            self.event_broadcaster.broadcast_group_change(event);
        }
        debug!("ContactsDb::add_group OK size: {}", size);
        Ok(())
    }

    pub fn update_group(&mut self, id: &str, name: &str) -> Result<(), Error> {
        debug!("ContactsDb::update_group id ={}, name= {}", id, name);
        let conn = self.db.connection();
        let size = conn.execute("UPDATE groups SET name = ? WHERE id = ?", &[name, id])?;
        if size > 0 {
            let event = GroupChangeEvent {
                reason: ChangeReason::Update,
                group: GroupInfo {
                    name: name.to_string(),
                    id: id.to_string(),
                },
            };
            self.event_broadcaster.broadcast_group_change(event);
        }
        debug!("ContactsDb::update_group OK size: {}", size);
        Ok(())
    }

    pub fn get_contactids_from_group(&mut self, group_id: &str) -> Result<Vec<String>, Error> {
        debug!(
            "ContactsDb::get_contactids_from_group group_id is :{}",
            group_id
        );
        let conn = self.db.connection();
        let mut stmt =
            conn.prepare("SELECT contact_id FROM group_contacts WHERE group_id IS :group_id")?;
        let rows = stmt.query_map(&[group_id], |row| row.get(0))?;

        Ok(rows_to_vec(rows))
    }

    pub fn get_all_groups(&mut self) -> Result<Vec<GroupInfo>, Error> {
        debug!("ContactsDb::get_all_groups");
        let conn = self.db.connection();
        let mut stmt = conn.prepare("SELECT * FROM groups ORDER BY name COLLATE NOCASE ASC")?;

        let rows = stmt.query_map(NO_PARAMS, |row| {
            Ok(GroupInfo {
                name: row.get(1)?,
                id: row.get(0)?,
            })
        })?;

        Ok(rows_to_vec(rows))
    }

    pub fn import_sim_contacts(&mut self, sim_contacts: &[SimContactInfo]) -> Result<(), Error> {
        debug!("ContactsDb::import_sim_contacts");
        let mut updated_contacts = vec![];
        // Holding the sim contact hash from the sim_contact_hash table.
        // Key: sim contact id, value: hash of this sim contact.
        let mut map: HashMap<String, String> = HashMap::new();
        let conn = self.db.connection();
        {
            let mut stmt = conn.prepare("SELECT id, hash FROM sim_contact_hash")?;
            let rows = stmt.query_map(NO_PARAMS, |row| {
                Ok(SimContactHash {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })?;

            for row in rows {
                let data = row?;
                map.insert(data.key, data.value);
            }

            for contact in sim_contacts {
                if let Some(value) = map.get(&contact.id) {
                    let hash_value = hash(&contact).to_string();

                    if &hash_value != value {
                        // Sim contacts need to update.
                        updated_contacts.push(contact.clone());
                    }

                    // Remove the sim contacts which need update or none change from map.
                    // Then the rest of the map is the contacts that should be delete.
                    map.remove(&contact.id);
                } else {
                    // New sim contacts.
                    updated_contacts.push(contact.clone());
                }
            }
        }

        let mut removed_ids = vec![];
        for key in map.keys() {
            removed_ids.push(key.clone());
        }
        if !removed_ids.is_empty() {
            self.remove(&removed_ids)?;
        }
        let contacts_info: Vec<ContactInfo> =
            updated_contacts.iter().map(|item| item.into()).collect();
        self.save(&contacts_info, true)?;
        let event = SimContactLoadedEvent {
            remove_count: removed_ids.len() as i64,
            update_count: updated_contacts.len() as i64,
        };
        self.event_broadcaster.broadcast_sim_contact_loaded(event);

        Ok(())
    }

    pub fn log(&self) {
        self.event_broadcaster.log();
    }
}

fn rows_to_vec<I, R>(source: I) -> Vec<R>
where
    I: core::iter::Iterator<Item = Result<R, rusqlite::Error>>,
{
    source
        .filter_map(|item| match item {
            Ok(val) => Some(val),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn create_contacts_database() {
        let _ = env_logger::try_init();

        let broadcaster = ContactsFactoryEventBroadcaster::default();

        let mut db = ContactsDb::new(broadcaster);
        db.clear_contacts().unwrap();
        assert_eq!(db.count().unwrap(), 0);

        if let Err(error) =
            load_contacts_to_db("./test-fixtures/contacts.json", db.db.mut_connection())
        {
            debug!(
                "load_contacts_to_db ./test-fixtures/contacts.json error {}",
                error
            );
        }
        assert_eq!(db.count().unwrap(), 2);
        db.clear_contacts().unwrap();
        assert_eq!(db.count().unwrap(), 0);

        if let Err(error) = load_contacts_to_db(
            "./test-fixtures/contacts_incorrect.json",
            db.db.mut_connection(),
        ) {
            debug!(
                "load_contacts_to_db ./test-fixtures/contacts_incorrect.json error {}",
                error
            );
        }
        assert_eq!(db.count().unwrap(), 0);

        let bob = ContactInfo {
            name: Some("Bob".to_string()),
            ..Default::default()
        };

        let alice = ContactInfo {
            name: Some("alice".to_string()),
            ..Default::default()
        };

        db.save(&[bob, alice], false).unwrap();

        assert_eq!(db.count().unwrap(), 2);

        db.clear_contacts().unwrap();

        assert_eq!(db.count().unwrap(), 0);

        // Import sim contacts.
        let sim_contact_1 = SimContactInfo {
            id: "0001".to_string(),
            tel: "13682628272\u{001E}18812345678\u{001E}19922223333".to_string(),
            email: "test@163.com\u{001E}happy@sina.com\u{001E}3179912@qq.com".to_string(),
            name: "Ted".to_string(),
            category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
        };

        let sim_contact_2 = SimContactInfo {
            id: "0002".to_string(),
            tel: "15912345678\u{001E}18923456789".to_string(),
            email: "test1@kaiostech.com\u{001E}231678456@qq.com".to_string(),
            name: "Bob".to_string(),
            category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
        };

        db.import_sim_contacts(&[sim_contact_1, sim_contact_2])
            .unwrap();

        assert_eq!(db.count().unwrap(), 2);

        if let Ok(contact) = db.get(&"0001".to_string(), true) {
            // Verify import sim_contact_1's data sucessful.
            assert_eq!(contact.name.unwrap(), "Ted");
            let tel = contact.tel.unwrap();
            assert_eq!(tel[0].value, "13682628272".to_string());
            assert_eq!(tel[1].value, "18812345678".to_string());
            assert_eq!(tel[2].value, "19922223333".to_string());
            let email = contact.email.unwrap();
            assert_eq!(email[0].value, "test@163.com".to_string());
            assert_eq!(email[1].value, "happy@sina.com".to_string());
            assert_eq!(email[2].value, "3179912@qq.com".to_string());
        }

        // Used to verify sim contact name changed.
        let sim_contact_1_name_change = SimContactInfo {
            id: "0001".to_string(),
            tel: "13682628272\u{001E}18812345678\u{001E}19922223333".to_string(),
            email: "test@163.com\u{001E}happy@sina.com\u{001E}3179912@qq.com".to_string(),
            name: "Jack".to_string(),
            category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
        };

        db.import_sim_contacts(&[sim_contact_1_name_change])
            .unwrap();

        if let Ok(contact) = db.get(&"0001".to_string(), true) {
            // Verify sim_contact_1's name update to "Jack".
            assert_eq!(contact.name.unwrap(), "Jack");
        }

        let mut cursor = db
            .get_all(
                ContactSortOptions {
                    sort_by: SortOption::Name,
                    sort_order: Order::Ascending,
                    sort_language: None,
                },
                10,
                true,
            )
            .unwrap();

        let contacts = cursor.next().unwrap();
        for contact in contacts {
            assert_eq!(contact.name.unwrap(), "Jack");
        }
        // Verify sim_contact_2 is removed.
        assert_eq!(db.count().unwrap(), 1);

        // To verify contact tel changed to 15229099710.
        let sim_contact_1_tel_change = SimContactInfo {
            id: "0001".to_string(),
            tel: "15229099710".to_string(),
            email: "test@163.com\u{001E}happy@sina.com\u{001E}3179912@qq.com".to_string(),
            name: "Jack".to_string(),
            category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
        };

        db.import_sim_contacts(&[sim_contact_1_tel_change]).unwrap();

        if let Ok(contact) = db.get(&"0001".to_string(), true) {
            // Verify sim_contact_1's tel update to "15229099710".
            assert_eq!(contact.tel.unwrap()[0].value, "15229099710".to_string());
        }

        // To verify contact email changed to zx@163.com.
        let sim_contact_1_email_change = SimContactInfo {
            id: "0001".to_string(),
            tel: "15229099710".to_string(),
            email: "zx@163.com".to_string(),
            name: "Jack".to_string(),
            category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
        };

        db.import_sim_contacts(&[sim_contact_1_email_change])
            .unwrap();

        if let Ok(contact) = db.get(&"0001".to_string(), true) {
            // Verify sim_contact_1's email update to "zx@163.com".
            assert_eq!(contact.email.unwrap()[0].value, "zx@163.com".to_string());
        }

        let sim_contacts = [
            SimContactInfo {
                id: "0001".to_string(),
                tel: "181".to_string(),
                email: "test@kaios.com".to_string(),
                name: "Are".to_string(),
                category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
            },
            SimContactInfo {
                id: "0002".to_string(),
                tel: "182".to_string(),
                email: "mbz@gmail.com".to_string(),
                name: "Bbc".to_string(),
                category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
            },
            SimContactInfo {
                id: "0003".to_string(),
                tel: "183".to_string(),
                email: "zx@kaiostech.com".to_string(),
                name: "David".to_string(),
                category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
            },
            SimContactInfo {
                id: "0004".to_string(),
                tel: "15229099710".to_string(),
                email: "test@163.com\u{001E}happy@sina.com\u{001E}3179912@qq.com".to_string(),
                name: "Zhang".to_string(),
                category: "KAICONTACT\u{001E}SIM0\u{001E}SIM".to_string(),
            },
        ];

        db.import_sim_contacts(&sim_contacts).unwrap();

        cursor = db
            .get_all(
                ContactSortOptions {
                    sort_by: SortOption::Name,
                    sort_order: Order::Ascending,
                    sort_language: None,
                },
                10,
                true,
            )
            .unwrap();

        let contacts = cursor.next().unwrap();
        for i in 0..contacts.len() {
            assert_eq!(contacts[i].name.clone().unwrap(), sim_contacts[i].name);
        }

        db.import_sim_contacts(&[]).unwrap();

        // Verify db is empty after import empty sim contacts.
        assert_eq!(db.count().unwrap(), 0);

        db.clear_contacts().unwrap();

        assert_eq!(db.count().unwrap(), 0);

        // Load contacts from a vcf file.
        let input = std::fs::read_to_string("./test-fixtures/contacts_200.vcf").unwrap();
        debug!("Importing contacts from vcf.");
        let _count = db.import_vcf(&input).unwrap();
        assert_eq!(db.count().unwrap(), 200);
    }
}
