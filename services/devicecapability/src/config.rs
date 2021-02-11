/// Config interface for DeviceCapability
use android_utils::{AndroidProperties, AndroidPropertyError, PropertyGetter};
use common::traits::Service;
use common::JsonValue;
use geckobridge::service::GeckoBridgeService;
use geckobridge::state::PrefValue;
use log::{debug, error, info, warn};
use thiserror::Error;

#[cfg(target_os = "android")]
const DEFAULT_CONFIG: &str = "/system/b2g/defaults/devicecapability.json";
#[cfg(all(not(target_os = "android"), test))]
const DEFAULT_CONFIG: &str = "./devicecapability.json";
#[cfg(all(not(target_os = "android"), not(test)))]
const DEFAULT_CONFIG: &str = "../services/devicecapability/devicecapability.json";

/*
 * Example of devicecapability.json:
 * {
 *   "name1": {
 *     "default": 60000,
 *     "method": "android-property"
 *    },
 *   "name2": {
 *     "default": false,
 *     "method": "preference"
 *    },
 *   "hardware.memory": {
 *     "default": 0,
 *     "method": "hardware-memory"
 *    }
 * }
 */

const DEFAULT_KEY: &str = "default";
const METHOD_KEY: &str = "method";
const METHOD_PROP: &str = "android-property";
const METHOD_PREF: &str = "preference";
const METHOD_HW_MEM: &str = "hardware-memory";

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unrecognized name")]
    InvalidName,
    #[error("Unrecognized method")]
    InvalidMethod,
    #[error("Android property get error")]
    AndroidPropertyGetErr(#[from] AndroidPropertyError),
    #[error("Standard io error")]
    StdIOError(#[from] std::io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (Error::InvalidName, Error::InvalidName) => true,
            (Error::InvalidMethod, Error::InvalidMethod) => true,
            (Error::AndroidPropertyGetErr(e1), Error::AndroidPropertyGetErr(e2)) => e1 == e2,
            (Error::StdIOError(e1), Error::StdIOError(e2)) => e1.kind() == e2.kind(),
            (..) => false,
        }
    }
}

pub struct DeviceCapabilityConfig {
    json_value: serde_json::Value,
}

fn get_from_prop(name: &str, default: Option<&serde_json::Value>) -> Result<JsonValue, Error> {
    debug!("get_from_prop {}", name);

    match AndroidProperties::get(&name, "") {
        Ok(value) => match value.as_ref() {
            "" => {
                debug!("empty");
                let safe_default = default.unwrap_or(&serde_json::Value::Null);
                Ok(JsonValue::from(safe_default.clone()))
            }
            "true" => {
                debug!("bool true");
                Ok(JsonValue::from(serde_json::Value::Bool(true)))
            }
            "false" => {
                debug!("bool false");
                Ok(JsonValue::from(serde_json::Value::Bool(false)))
            }
            s => {
                if s.chars().all(char::is_numeric) {
                    debug!("number {}", s);
                    match s.parse::<i64>() {
                        Ok(num) => Ok(JsonValue::from(serde_json::Value::Number(num.into()))),
                        Err(e) => {
                            debug!(
                                "parse number error. return as string. {} {} {:?}",
                                name, s, e
                            );
                            Ok(JsonValue::from(serde_json::Value::String(value)))
                        }
                    }
                } else {
                    debug!("string {}", s);
                    Ok(JsonValue::from(serde_json::Value::String(value)))
                }
            }
        },
        Err(e) => {
            error!("AndroidProperties::get error. {} {:?}", name, e);
            Err(Error::AndroidPropertyGetErr(e))
        }
    }
}

fn get_from_pref(name: &str, default: Option<&serde_json::Value>) -> JsonValue {
    debug!("get_from_pref {}", name);
    let bridge = GeckoBridgeService::shared_state();
    let result = bridge.lock().get_pref(name);
    match result {
        Some(PrefValue::Bool(value)) => JsonValue::from(serde_json::Value::Bool(value)),
        Some(PrefValue::Int(value)) => JsonValue::from(serde_json::Value::Number(value.into())),
        Some(PrefValue::Str(value)) => JsonValue::from(serde_json::Value::String(value)),
        None => JsonValue::from(json!(default.unwrap_or(&serde_json::Value::Null))),
    }
}

pub fn read_config(config_path: &str) -> serde_json::Value {
    match std::fs::File::open(config_path) {
        Ok(config_file) => match serde_json::from_reader(config_file) {
            Ok(value) => {
                debug!("read {:?} from {}", value, config_path);
                value
            }
            Err(e) => {
                error!("read config file {} error {:?}", config_path, e);
                json!({})
            }
        },
        Err(e) => {
            error!("open config file {} error {:?}", config_path, e);
            json!({})
        }
    }
}

impl Default for DeviceCapabilityConfig {
    fn default() -> Self {
        let config_path = match std::env::var("DEVICE_CAPABILITY_CONFIG") {
            Ok(val) => val,
            Err(_) => DEFAULT_CONFIG.to_string(),
        };

        info!("import from {}", config_path);
        Self {
            json_value: read_config(&config_path),
        }
    }
}

impl DeviceCapabilityConfig {
    pub fn len(&self) -> usize {
        match self.json_value.as_object() {
            Some(obj) => obj.len(),
            None => 0,
        }
    }

    pub fn get(&self, name: &str) -> Result<JsonValue, Error> {
        match self.json_value.get(name) {
            Some(val) => {
                info!("config get name:{} val: {}", name, val);
                match val.get(METHOD_KEY) {
                    Some(method) => match method.as_str() {
                        Some(METHOD_PROP) => get_from_prop(name, val.get(DEFAULT_KEY)),
                        Some(METHOD_PREF) => Ok(get_from_pref(name, val.get(DEFAULT_KEY))),
                        Some(METHOD_HW_MEM) => {
                            debug!("get android_utils::total_memory");
                            Ok(JsonValue::from(serde_json::Value::Number(
                                android_utils::total_memory().into(),
                            )))
                        }
                        Some(m) => {
                            error!("Unrecognized method. {}", m);
                            Err(Error::InvalidMethod)
                        }
                        None => Ok(JsonValue::from(
                            val.get(DEFAULT_KEY).unwrap_or(&json!(false)).clone(),
                        )),
                    },
                    None => {
                        warn!(
                            "Method is not specified, return default. {} {:?}",
                            name, val
                        );
                        Ok(JsonValue::from(
                            val.get(DEFAULT_KEY).unwrap_or(&json!(false)).clone(),
                        ))
                    }
                }
            }
            None => {
                error!(
                    "No such name in config, return Error::InvalidName. {}",
                    name
                );
                Err(Error::InvalidName)
            }
        }
    }
}

#[test]
fn test_import_devicecapability() {
    let config = DeviceCapabilityConfig::default();
    assert!(config.len() > 0);
}

#[test]
fn test_basic_calls() {
    use serde_json::Value;
    let config = DeviceCapabilityConfig::default();

    assert_eq!(*config.get("device.bt").unwrap(), Value::Bool(true));
    assert_eq!(
        *config.get("ro.build.type").unwrap(),
        Value::String("".into())
    );
    assert!(config.get("hardware.memory").unwrap().as_i64().unwrap() > 0);
}

#[test]
fn test_unrecognized_name() {
    let config = DeviceCapabilityConfig::default();
    assert_eq!(
        config.get("this-is-an-invalid-name").unwrap_err(),
        Error::InvalidName
    );
}
