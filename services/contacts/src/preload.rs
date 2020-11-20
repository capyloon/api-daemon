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
    #[error("SQlite error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serde JSON error")]
    Json(#[from] serde_json::Error),
    #[error("Parse Time Error")]
    ParseTime(#[from] chrono::format::ParseError),
    #[error("Time Error")]
    Time(String),
    #[error("IO Error")]
    IO(#[from] std::io::Error),
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
            atype: String::new(),
            value: String::new(),
            pref: false,
        }
    }
}

impl Default for ContactTelField {
    fn default() -> Self {
        ContactTelField {
            atype: String::new(),
            value: String::new(),
            pref: false,
            carrier: String::new(),
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Address {
            atype: String::new(),
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
        let mut contact_field = ContactField::default();

        if let Some(atype) = &json_contact_field.atype {
            contact_field.atype = atype.to_string();
        }
        if let Some(value) = &json_contact_field.value {
            contact_field.value = value.to_string();
        }
        if let Some(pref) = &json_contact_field.pref {
            contact_field.pref = *pref;
        }

        contact_field
    }
}

impl From<&JsonContactTelField> for ContactTelField {
    fn from(json_tel_field: &JsonContactTelField) -> Self {
        let mut contact_tel_field = ContactTelField::default();

        if let Some(atype) = &json_tel_field.atype {
            contact_tel_field.atype = atype.to_string();
        }
        if let Some(value) = &json_tel_field.value {
            contact_tel_field.value = value.to_string();
        }
        if let Some(pref) = &json_tel_field.pref {
            contact_tel_field.pref = *pref;
        }
        if let Some(carrier) = &json_tel_field.carrier {
            contact_tel_field.carrier = carrier.to_string();
        }

        contact_tel_field
    }
}

impl From<&JsonAddress> for Address {
    fn from(json_address: &JsonAddress) -> Self {
        let mut address = Address::default();

        if let Some(atype) = &json_address.atype {
            address.atype = atype.to_string();
        }
        if let Some(street_address) = &json_address.street_address {
            address.street_address = Some(street_address.to_string());
        }
        if let Some(locality) = &json_address.locality {
            address.locality = Some(locality.to_string());
        }
        if let Some(region) = &json_address.region {
            address.region = Some(region.to_string());
        }
        if let Some(postal_code) = &json_address.postal_code {
            address.postal_code = Some(postal_code.to_string());
        }
        if let Some(country_name) = &json_address.country_name {
            address.country_name = Some(country_name.to_string());
        }
        if let Some(pref) = &json_address.pref {
            address.pref = Some(*pref);
        }

        address
    }
}

impl From<&JsonContactInfo> for ContactInfo {
    fn from(json_contact: &JsonContactInfo) -> Self {
        let mut contact = ContactInfo::default();

        if let Some(sex) = &json_contact.sex {
            contact.sex = sex.to_string();
        }

        if let Some(gender_identity) = &json_contact.gender_identity {
            contact.gender_identity = gender_identity.to_string();
        }

        if let Some(ringtone) = &json_contact.ringtone {
            contact.ringtone = ringtone.to_string();
        }

        if let Some(photo_type) = &json_contact.photo_type {
            contact.photo_type = photo_type.to_string();
        }

        if let Some(name) = &json_contact.name {
            contact.name = name.to_string();
        }

        if let Some(given_name) = &json_contact.given_name {
            contact.given_name = given_name.to_string();
        }

        if let Some(phonetic_given_name) = &json_contact.phonetic_given_name {
            contact.phonetic_given_name = phonetic_given_name.to_string();
        }

        if let Some(family_name) = &json_contact.family_name {
            contact.family_name = family_name.to_string();
        }

        if let Some(phonetic_family_name) = &json_contact.phonetic_family_name {
            contact.phonetic_family_name = phonetic_family_name.to_string();
        }

        if let Some(time_str) = json_contact.bday.as_ref() {
            if let Ok(time) = json_string_to_systemtime(time_str.to_string()) {
                contact.bday = time;
            }
        }

        if let Some(time_str) = json_contact.anniversary.as_ref() {
            if let Ok(time) = json_string_to_systemtime(time_str.to_string()) {
                contact.anniversary = time;
            }
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

        contact.honorific_prefix = json_contact.honorific_prefix.clone();
        contact.additional_name = json_contact.additional_name.clone();
        contact.honorific_suffix = json_contact.honorific_suffix.clone();
        contact.nickname = json_contact.nickname.clone();
        contact.category = json_contact.category.clone();
        contact.org = json_contact.org.clone();
        contact.job_title = json_contact.job_title.clone();
        contact.note = json_contact.note.clone();
        contact.groups = json_contact.groups.clone();

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

        contact.id = Uuid::new_v4().to_string();
        contact.published = SystemTime::from(std::time::SystemTime::now());
        debug!("save current contact id is {}", contact.id);

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
