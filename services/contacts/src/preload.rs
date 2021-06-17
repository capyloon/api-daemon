use crate::generated::common::*;
use chrono::{self, NaiveDateTime};
use common::SystemTime;
use log::{debug, error};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::time::{Duration, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

pub const CONTACTS_PRELOAD_FILE_PATH: &str = "/system/b2g/defaults/contacts.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("SQlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serde JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Parse Time Error: {0}")]
    ParseTime(#[from] chrono::format::ParseError),
    #[error("Time Error")]
    Time(String),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Deserialize, Debug)]
pub struct JsonAddress {
    pub atype: Option<String>,
    pub street_address: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_name: Option<String>,
    pub pref: Option<bool>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct JsonContactField {
    pub atype: Option<String>,
    pub value: Option<String>,
    pub pref: Option<bool>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct JsonContactTelField {
    pub atype: Option<String>,
    pub value: Option<String>,
    pub pref: Option<bool>,
    pub carrier: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct JsonContactInfo {
    pub bday: Option<String>,
    pub anniversary: Option<String>,
    pub sex: Option<String>,
    pub gender_identity: Option<String>,
    pub ringtone: Option<String>,
    pub photo_type: Option<String>,
    pub photo_blob: Option<Value>,
    pub addresses: Option<Vec<JsonAddress>>,
    pub email: Option<Vec<JsonContactField>>,
    pub url: Option<Vec<JsonContactField>>,
    pub name: Option<String>,
    pub tel: Option<Vec<JsonContactTelField>>,
    pub honorific_prefix: Option<Vec<String>>,
    pub given_name: Option<String>,
    pub phonetic_given_name: Option<String>,
    pub additional_name: Option<Vec<String>>,
    pub family_name: Option<String>,
    pub phonetic_family_name: Option<String>,
    pub honorific_suffix: Option<Vec<String>>,
    pub nickname: Option<Vec<String>>,
    pub category: Option<Vec<String>>,
    pub org: Option<Vec<String>>,
    pub job_title: Option<Vec<String>>,
    pub note: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

impl Default for ContactField {
    fn default() -> Self {
        ContactField {
            atype: None,
            value: String::new(),
            pref: Some(false),
        }
    }
}

impl Default for ContactTelField {
    fn default() -> Self {
        ContactTelField {
            atype: None,
            value: String::new(),
            pref: Some(false),
            carrier: None,
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Address {
            atype: None,
            street_address: None,
            locality: None,
            region: None,
            postal_code: None,
            country_name: None,
            pref: None,
        }
    }
}

fn json_string_to_systemtime(time_str: String) -> Result<SystemTime, Error> {
    debug!("json_string_to_systemtime from time_str: {}", time_str);

    let date_time = NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S")?;
    let time = UNIX_EPOCH
        .checked_add(Duration::from_secs(date_time.timestamp() as u64))
        .ok_or_else(|| Error::Time("parse time error".into()))?;
    Ok(SystemTime::from(time))
}

impl From<&JsonContactField> for ContactField {
    fn from(json_contact_field: &JsonContactField) -> Self {
        ContactField {
            atype: json_contact_field.atype.clone(),
            value: json_contact_field.value.clone().unwrap_or_default(),
            pref: json_contact_field.pref,
        }
    }
}

impl From<&JsonContactTelField> for ContactTelField {
    fn from(json_tel_field: &JsonContactTelField) -> Self {
        let mut contact_tel_field = ContactTelField {
            pref: json_tel_field.pref,
            carrier: json_tel_field.carrier.clone(),
            ..Default::default()
        };
        if let Some(value) = &json_tel_field.value {
            contact_tel_field.value = value.into();
        }

        contact_tel_field
    }
}

impl From<&JsonAddress> for Address {
    fn from(json_address: &JsonAddress) -> Self {
        Address {
            atype: json_address.atype.clone(),
            street_address: json_address.street_address.clone(),
            locality: json_address.locality.clone(),
            region: json_address.region.clone(),
            postal_code: json_address.postal_code.clone(),
            country_name: json_address.country_name.clone(),
            pref: json_address.pref,
        }
    }
}

impl From<&JsonContactInfo> for ContactInfo {
    fn from(json_contact: &JsonContactInfo) -> Self {
        let mut contact = ContactInfo {
            sex: json_contact.sex.clone(),
            gender_identity: json_contact.gender_identity.clone(),
            ringtone: json_contact.ringtone.clone(),
            photo_type: json_contact.photo_type.clone(),
            name: json_contact.name.clone(),
            given_name: json_contact.given_name.clone(),
            phonetic_given_name: json_contact.phonetic_given_name.clone(),
            family_name: json_contact.family_name.clone(),
            phonetic_family_name: json_contact.phonetic_family_name.clone(),
            honorific_prefix: json_contact.honorific_prefix.clone(),
            additional_name: json_contact.additional_name.clone(),
            honorific_suffix: json_contact.honorific_suffix.clone(),
            nickname: json_contact.nickname.clone(),
            category: json_contact.category.clone(),
            org: json_contact.org.clone(),
            job_title: json_contact.job_title.clone(),
            note: json_contact.note.clone(),
            groups: json_contact.groups.clone(),
            ..Default::default()
        };

        if let Some(time_str) = json_contact.bday.as_ref() {
            contact.bday = json_string_to_systemtime(time_str.to_string()).ok();
        }

        if let Some(time_str) = json_contact.anniversary.as_ref() {
            contact.anniversary = json_string_to_systemtime(time_str.to_string()).ok();
        }

        let mut email_array: Vec<ContactField> = Vec::new();

        if let Some(json_email_array) = json_contact.email.as_ref() {
            for json_email_item in json_email_array {
                email_array.push(json_email_item.into());
            }
            contact.email = Some(email_array);
        }

        let mut url_array: Vec<ContactField> = Vec::new();

        if let Some(json_url_array) = json_contact.url.as_ref() {
            for json_url_item in json_url_array {
                url_array.push(json_url_item.into());
            }
            contact.url = Some(url_array);
        }

        let mut tel_array: Vec<ContactTelField> = Vec::new();

        if let Some(json_tel_array) = json_contact.tel.as_ref() {
            for json_tel_item in json_tel_array {
                tel_array.push(json_tel_item.into());
            }
            contact.tel = Some(tel_array);
        }

        let mut addresses_array: Vec<Address> = Vec::new();
        if let Some(json_addresses_array) = json_contact.addresses.as_ref() {
            for json_address_item in json_addresses_array {
                addresses_array.push(json_address_item.into());
            }
            contact.addresses = Some(addresses_array);
        }

        contact
    }
}

fn import_contacts_to_db(
    connection: &mut Connection,
    contacts: &[ContactInfo],
) -> Result<(), Error> {
    for contact_info in contacts {
        let tx = connection.transaction()?;
        let mut contact = contact_info.clone();

        contact.id = Some(Uuid::new_v4().to_string());
        contact.published = Some(SystemTime::from(std::time::SystemTime::now()));
        debug!("save current contact id is {:?}", contact.id);

        if let Err(err) = contact.save_main_data(&tx) {
            error!("save_main_data error: {}, continue", err);
            continue;
        }
        if let Err(err) = contact.save_additional_data(&tx) {
            error!("save_additional_data error: {}, continue", err);
            continue;
        }

        tx.commit()?;
    }

    Ok(())
}

pub fn load_contacts_to_db(file_path: &str, connection: &mut Connection) -> Result<(), Error> {
    debug!("load_contacts_to_db start");

    let file = File::open(file_path)?;
    let json_contacts: Vec<JsonContactInfo> = serde_json::from_reader(file)?;
    debug!("load_contacts_to_db got json_contacts: {:?}", json_contacts);

    let contacts: Vec<ContactInfo> = json_contacts.iter().map(|item| item.into()).collect();
    import_contacts_to_db(connection, &contacts)?;

    Ok(())
}
