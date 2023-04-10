use crate::interface::InterfaceType;
use std::convert::TryFrom;
use std::fs::read_to_string;

pub fn get_interface_type(if_name: String) -> InterfaceType {
    let if_type_path: String = format!("/sys/class/net/{}/type", if_name);
    let r = read_to_string(if_type_path);
    match r {
        Ok(content) => {
            let if_type_string = content.trim().to_string();
            match if_type_string.parse::<u32>() {
                Ok(if_type) => {
                    return InterfaceType::try_from(if_type).unwrap_or(InterfaceType::Unknown);
                }
                Err(_) => {
                    return InterfaceType::Unknown;
                }
            }
        }
        Err(_) => {
            return InterfaceType::Unknown;
        }
    };
}

pub fn get_interface_speed(if_name: String) -> Option<u64> {
    let if_speed_path: String = format!("/sys/class/net/{}/speed", if_name);
    let r = read_to_string(if_speed_path);
    match r {
        Ok(content) => {
            let if_speed_string = content.trim().to_string();
            match if_speed_string.parse::<u64>() {
                Ok(if_speed) => {
                    // Convert Mbps to bps
                    return Some(if_speed * 1000000);
                }
                Err(_) => {
                    return None;
                }
            }
        }
        Err(_) => {
            return None;
        }
    };
}
