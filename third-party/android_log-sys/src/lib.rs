// Copyright 2016 The android_log_sys Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::os::raw;

#[allow(non_camel_case_types)]
pub type c_va_list = raw::c_void;
#[allow(non_camel_case_types)]
pub type c_int = raw::c_int;
#[allow(non_camel_case_types)]
pub type c_char = raw::c_char;

// automatically generated by rust-bindgen

#[derive(Clone, Copy)]
#[repr(isize)]
pub enum LogPriority {
    UNKNOWN = 0,
    DEFAULT = 1,
    VERBOSE = 2,
    DEBUG = 3,
    INFO = 4,
    WARN = 5,
    ERROR = 6,
    FATAL = 7,
    SILENT = 8,
}

#[link(name = "log")]
extern "C" {
    pub fn __android_log_write(prio: c_int,
                               tag: *const c_char,
                               text: *const c_char)
                               -> c_int;
    pub fn __android_log_print(prio: c_int,
                               tag: *const c_char,
                               fmt: *const c_char,
                               ...)
                               -> c_int;
    pub fn __android_log_vprint(prio: c_int,
                                tag: *const c_char,
                                fmt: *const c_char,
                                ap: *mut c_va_list)
                                -> c_int;
    pub fn __android_log_assert(cond: *const c_char,
                                tag: *const c_char,
                                fmt: *const c_char,
                                ...);
}
