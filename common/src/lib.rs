use bincode::Options;
use log::info;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value as SerdeValue;
use std::fmt;
use std::ops::Deref;
use std::time::UNIX_EPOCH;
use threadpool::ThreadPool;
use traits::{EventMapKey, SharedEventMap};

pub mod build_helper;
pub mod core;
pub mod device_info;
pub mod frame;
pub mod object_tracker;
pub mod remotable;
pub mod remote_service;
pub mod remote_services_registrar;
mod selinux;
pub mod socket_pair;
pub mod tokens;
#[macro_use]
pub mod traits;
pub mod observers;

pub use bincode::Error as BincodeError;

pub fn get_bincode() -> impl Options {
    bincode::options().with_big_endian().with_varint_encoding()
}

pub fn deserialize_bincode<'a, T>(input: &'a [u8]) -> Result<T, BincodeError>
where
    T: Deserialize<'a>,
{
    get_bincode().deserialize(input)
}

pub fn is_event_in_map(map: &SharedEventMap, service: u32, object: u32, event: u32) -> bool {
    let res = match map.lock().get(&EventMapKey::new(service, object, event)) {
        Some(&true) => true,
        Some(&false) => false,
        None => false,
    };

    info!(
        "Checking event service #{} object #{} event #{} : {}",
        service, object, event, res
    );

    res
}

pub fn threadpool_status(pool: &ThreadPool) -> String {
    format!(
        "size: {}, active: {}, queued: {}",
        pool.max_count(),
        pool.active_count(),
        pool.queued_count(),
    )
}

// A wrapper around a JsonValue to help with the encoding/decoding of JSON strings.
#[derive(Clone, Debug)]
pub struct JsonValue(SerdeValue);

impl<'de> Deserialize<'de> for JsonValue {
    fn deserialize<D>(deserializer: D) -> Result<JsonValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        use std::str::FromStr;

        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a JSON string")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(s.to_owned())
            }
        }

        let json_str = deserializer.deserialize_str(JsonVisitor)?;
        let value =
            SerdeValue::from_str(&json_str).map_err(|err| D::Error::custom(format!("{}", err)))?;

        Ok(JsonValue(value))
    }
}

impl Serialize for JsonValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Deref for JsonValue {
    type Target = SerdeValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SerdeValue> for JsonValue {
    fn from(v: SerdeValue) -> Self {
        JsonValue(v)
    }
}

// A wrapper around a std::time::SystemTime to provide serde support as u64 milliseconds since epoch.
#[derive(Clone, Debug)]
pub struct SystemTime(std::time::SystemTime);

impl<'de> Deserialize<'de> for SystemTime {
    fn deserialize<D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeVisitor;
        impl<'de> Visitor<'de> for TimeVisitor {
            type Value = i64;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "i64: time in ms since epoch")
            }

            fn visit_i64<E>(self, val: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(val)
            }
        }

        let milliseconds = deserializer.deserialize_i64(TimeVisitor)?;
        let system_time = if milliseconds >= 0 {
            UNIX_EPOCH
                .checked_add(std::time::Duration::from_millis(milliseconds as _))
                .unwrap_or(UNIX_EPOCH)
        } else {
            UNIX_EPOCH
                .checked_sub(std::time::Duration::from_millis(-milliseconds as _))
                .unwrap_or(UNIX_EPOCH)
        };
        Ok(SystemTime(system_time))
    }
}

impl Serialize for SystemTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0.duration_since(UNIX_EPOCH) {
            Ok(from_epoch) => serializer.serialize_i64(from_epoch.as_millis() as _),
            // In the error case, we get the number of milliseconds as the error duration.
            Err(err) => serializer.serialize_i64(-(err.duration().as_millis() as i64)),
        }
    }
}

impl Deref for SystemTime {
    type Target = std::time::SystemTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<std::time::SystemTime> for SystemTime {
    fn from(v: std::time::SystemTime) -> Self {
        SystemTime(v)
    }
}

// The type used to represent Blobs on the Rust side.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Blob {
    mime_type: String,
    data: Vec<u8>,
}

impl Blob {
    pub fn new(mime_type: &str, data: Vec<u8>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn mime_type(&self) -> String {
        self.mime_type.clone()
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn take_data(self) -> Vec<u8> {
        self.data
    }

    pub fn set_mime_type(&mut self, mime_type: &str) {
        self.mime_type = mime_type.into();
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }
}

// Re-export url::Url to make it available to generated code.
pub use url::Url;

// This allows writing log into a file with limited lines.
// It will append the log to the end of the file.
// When the number of lines reaches the limit,
// it will rotate into a new file per the file limit.
// log_file - the path of the log file.
// file_limit - Maximum number of rotated files, 0 for a single file.
// line_limit - Maximum number of lines per file.
// log_line - one line of log.
#[macro_export]
macro_rules! log_warning {
    ($log_file:expr, $file_limit:expr, $line_limit:expr, $log_line:path) => {
        use file_rotate::{
            compression::Compression, suffix::AppendCount, ContentLimit, FileRotate,
        };
        use std::io::Write;
        use std::path::Path;
        let log_path = Path::new($log_file);
        let mut log = FileRotate::new(
            log_path.clone(),
            AppendCount::new($file_limit),
            ContentLimit::Lines($line_limit),
            Compression::None,
        );
        let _ = writeln!(log, "{}", $log_line);
        warn!("{}", $log_line);
    };
}
