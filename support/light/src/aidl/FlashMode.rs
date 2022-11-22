#![forbid(unsafe_code)]
// #![rustfmt::skip]
#![allow(non_upper_case_globals)]
use binder::declare_binder_enum;
declare_binder_enum! {
  r#FlashMode : [i8; 3] {
    r#NONE = 0,
    r#TIMED = 1,
    r#HARDWARE = 2,
  }
}
pub(crate) mod mangled {
 pub use super::r#FlashMode as _7_android_8_hardware_5_light_9_FlashMode;
}
