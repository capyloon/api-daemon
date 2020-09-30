use regex::Regex;
use serde;
use serde_json;
use std;

lazy_static! {
    static ref MIN_REGEX: Regex = Regex::new(r"[\n\t]|\s{4}").unwrap();
}

pub fn check_serialize_deserialize<T>(json: &str, data: &T)
where
    T: std::fmt::Debug,
    T: std::cmp::PartialEq,
    T: serde::de::DeserializeOwned,
    T: serde::Serialize,
{
    let min_json = MIN_REGEX.replace_all(json, "");

    assert_eq!(*data, serde_json::from_str::<T>(&min_json).unwrap());
    assert_eq!(min_json, serde_json::to_string(data).unwrap());
}

pub fn check_deserialize<T>(json: &str, data: &T)
where
    T: std::fmt::Debug,
    T: std::cmp::PartialEq,
    T: serde::de::DeserializeOwned,
{
    let min_json = MIN_REGEX.replace_all(json, "");

    assert_eq!(serde_json::from_str::<T>(&min_json).unwrap(), *data);
}

pub fn check_serialize<T>(json: &str, data: &T)
where
    T: std::fmt::Debug,
    T: std::cmp::PartialEq,
    T: serde::Serialize,
{
    let min_json = MIN_REGEX.replace_all(json, "");

    assert_eq!(min_json, serde_json::to_string(data).unwrap());
}
