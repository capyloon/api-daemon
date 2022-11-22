#![forbid(unsafe_code)]
// #![rustfmt::skip]
#![allow(non_upper_case_globals)]
use binder::declare_binder_enum;
declare_binder_enum! {
  r#LightType : [i8; 10] {
    r#BACKLIGHT = 0,
    r#KEYBOARD = 1,
    r#BUTTONS = 2,
    r#BATTERY = 3,
    r#NOTIFICATIONS = 4,
    r#ATTENTION = 5,
    r#BLUETOOTH = 6,
    r#WIFI = 7,
    r#MICROPHONE = 8,
    r#CAMERA = 9,
  }
}
pub(crate) mod mangled {
 pub use super::r#LightType as _7_android_8_hardware_5_light_9_LightType;
}
