#[macro_use]
pub mod generated;
#[macro_use]
extern crate log;

use generated::ffi::root;
use std::os::raw::c_void;
use std::ffi::{CString, CStr};

extern "C" fn dump_callback(
    descriptor: *const c_void,
    _context: *mut c_void,
    succeeded: bool,
) -> bool {
    // Dump call back from exception
    let path: &CStr = unsafe { CStr::from_ptr(root::rust_breakpad_descriptor_path(descriptor)) };
    debug!("Crash! Dump to file: {:?}\n", path);
    return succeeded;
}

extern "C" fn filter_callback(_context: *mut c_void) -> bool {
    // Filter call back from exception
    // TODO: add some stamps from kaios.
    return true;
}

type ExceptionHandler = usize;

pub fn init_breakpad(path: String) -> ExceptionHandler {
    let mut content = vec![0];
    unsafe {
        let cpath = CString::new(path.into_bytes()).unwrap();
        let descriptor =
            root::rust_breakpad_descriptor_new(cpath.as_ptr());
        let exception_handler = root::rust_breakpad_exceptionhandler_new(
            descriptor,
            filter_callback as *mut c_void,
            dump_callback as *mut c_void,
            content.as_mut_ptr() as *mut c_void,
            -1,
        );
        // Rust forbid send raw pointer between thread directly because of
        // negative impl for c_void: impl<T> !Send for *mut T
        // Have had to convert raw ptr as usize to avoid compiler check.
        std::mem::transmute::<*mut c_void, ExceptionHandler>(exception_handler)
    }
}

pub fn write_minidump(exception_handler: ExceptionHandler) -> bool {
    unsafe {
        let ptr = std::mem::transmute::<ExceptionHandler, *mut c_void>(exception_handler);
        root::rust_breakpad_exceptionhandler_write_minidump(ptr)
    }
}

#[cfg(test)]
mod test {
    use super::init_breakpad;
    use super::write_minidump;
    use std::fs;

    #[test]
    fn breakpad_start() {
        let _ = fs::create_dir("./tmp");
        match fs::metadata("./tmp") {
            Ok(metadata) => assert!(!metadata.permissions().readonly()),
            Err(e) => panic!("no folder created!{:?}", e),
        }
        let exception_handler = init_breakpad("./tmp".into());
        assert_ne!(exception_handler, 0);
        // TODO: Always failed on CI server, waiting for debug
        let succ = write_minidump(exception_handler);
        println!("write dump: {}", succ);
        
        // Show all dump files under tmp folder
        match fs::read_dir("./tmp") {
            Ok(read) => {
                for entry in read {
                    match entry {
                        Ok(entry) => println!("file: {:?}", entry.path()),
                        Err(e) => panic!("no folder created!{:?}", e),
                    }
                }
            }
            Err(e) => panic!("no folder created!{:?}", e),
        }
        let _ = fs::remove_dir_all("./tmp");
    }
}
