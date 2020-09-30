// Generated by bindgen and pruned:
// bindgen $GONK_DIR/system/core/include/cutils/properties.h -- -I $GONK_DIR/bionic/libc/include/ --sysroot $GONK_DIR/prebuilts/ndk/9/platforms/android-21/arch-arm/ > ffi.rs

pub const PROP_NAME_MAX: usize = 32;
pub const PROP_VALUE_MAX: usize = 92;

// Link with libcutils.so
#[link(name = "cutils")]
extern "C" {
    // Returns the property length without the terminating NULL.
    pub fn property_get(
        key: *const ::std::os::raw::c_char,
        value: *mut ::std::os::raw::c_char,
        default_value: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;

    pub fn property_get_bool(key: *const ::std::os::raw::c_char, default_value: i8) -> i8;

    pub fn property_get_int64(key: *const ::std::os::raw::c_char, default_value: i64) -> i64;

    pub fn property_get_int32(key: *const ::std::os::raw::c_char, default_value: i32) -> i32;

    pub fn property_set(
        key: *const ::std::os::raw::c_char,
        value: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
