#![forbid(unsafe_code)]
// #![rustfmt::skip]
#![allow(non_upper_case_globals)]
use binder::declare_binder_enum;
declare_binder_enum! {
  r#BrightnessMode : [i8; 3] {
    r#USER = 0,
    r#SENSOR = 1,
    r#LOW_PERSISTENCE = 2,
  }
}
pub(crate) mod mangled {
 pub use super::r#BrightnessMode as _7_android_8_hardware_5_light_14_BrightnessMode;
}
