/* -*- Mode: rust; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* vim: set ts=8 sts=4 et sw=4 tw=80: */
#[cfg(target_os = "android")]
mod cutils {
    // A placeholder of Parcel defined in parcel.cpp
    #[repr(C)]
    pub struct CParcel {
        a: i32,
    }
    #[repr(C)]
    #[derive(Debug)]
    pub struct CNativeHandle {
        pub version: i32,
        pub num_fds: i32,
        pub num_ints: i32, // data[0] size == numFds + numInts
    }

    extern "C" {
        // Following functions are defined in parcel.cpp
        pub fn parcel_create() -> *mut CParcel;
        pub fn parcel_delete(p: *mut CParcel);

        pub fn parcel_data_size(p: *mut CParcel) -> usize;
        pub fn parcel_set_data_position(p: *mut CParcel, pos: usize);

        pub fn parcel_write_interface_token(p: *mut CParcel, iface: *const u8, len: usize) -> i32;

        pub fn parcel_write_int8(_p: *mut CParcel, _v: i8) -> i32;
        pub fn parcel_read_int8(_p: *mut CParcel, _v: *mut i8) -> i32;
        pub fn parcel_write_uint8(_p: *mut CParcel, _v: u8) -> i32;
        pub fn parcel_read_uint8(_p: *mut CParcel, _v: *mut u8) -> i32;

        pub fn parcel_write_int16(_p: *mut CParcel, _v: i16) -> i32;
        pub fn parcel_read_int16(_p: *mut CParcel, _v: *mut i16) -> i32;
        pub fn parcel_write_uint16(_p: *mut CParcel, _v: u16) -> i32;
        pub fn parcel_read_uint16(_p: *mut CParcel, _v: &mut u16) -> i32;

        pub fn parcel_write_int32(_p: *mut CParcel, _v: i32) -> i32;
        pub fn parcel_read_int32(_p: *mut CParcel, _v: *mut i32) -> i32;
        pub fn parcel_write_uint32(_p: *mut CParcel, _v: u32) -> i32;
        pub fn parcel_read_uint32(_p: *mut CParcel, _v: *mut u32) -> i32;

        pub fn parcel_write_int64(_p: *mut CParcel, _v: i64) -> i32;
        pub fn parcel_read_int64(_p: *mut CParcel, _v: *mut i64) -> i32;
        pub fn parcel_write_uint64(_p: *mut CParcel, _v: u64) -> i32;
        pub fn parcel_read_uin64(_p: *mut CParcel, _v: *mut u64) -> i32;

        pub fn parcel_write_float(_p: *mut CParcel, _v: f32) -> i32;
        pub fn parcel_read_float(_p: *mut CParcel, _v: *mut f32) -> i32;
        pub fn parcel_write_double(_p: *mut CParcel, _v: f64) -> i32;
        pub fn parcel_read_double(_p: *mut CParcel, _v: *mut f64) -> i32;

        pub fn parcel_read_buffer(
            p: *mut CParcel,
            buffer_size: usize,
            buffer_handle: *mut usize,
            buffer: *mut u8,
        ) -> i32;
        pub fn parcel_write_buffer(
            p: *mut CParcel,
            buffer: *const u8,
            buffer_size: usize,
            buffer_handle: *mut usize,
        ) -> i32;

        pub fn parcel_write_embedded_buffer(
            p: *mut CParcel,
            buffer: *const u8,
            buffer_size: usize,
            buffer_handle: *mut usize,
            parent_handle: usize,
            parent_offset: usize,
        ) -> i32;
        pub fn parcel_read_embedded_buffer(
            p: *mut CParcel,
            buffer_size: usize,
            buffer_handle: *mut usize,
            parent_handle: usize,
            parent_offset: usize,
            buffer: *mut u8,
        ) -> i32;

        pub fn parcel_write_native_handle_no_dup(
            p: *mut CParcel,
            handle: *const CNativeHandle,
            embedded: bool,
            parent_handle: usize,
            parent_offset: usize,
        ) -> i32;
        pub fn parcel_read_nullable_native_handle_no_dup(
            p: *mut CParcel,
            handle: *mut *const CNativeHandle,
            embedded: bool,
            parent_handle: usize,
            parent_offset: usize,
        ) -> i32;
    }

    // A placeholder of IBinderWrapper defined in ibinder.cpp
    #[repr(C)]
    pub struct CIBinder {
        a: i32,
    }

    extern "C" {
        // Following functions are defined in ibinder.cpp
        pub fn ibinder_query_ibinder(
            iface: *const u8,
            iface_sz: usize,
            service_name: *const u8,
            service_name_sz: usize,
        ) -> *mut CIBinder;
        pub fn ibinder_delete(ibinder: *mut CIBinder);
        pub fn ibinder_transact(
            ibinder: *mut CIBinder,
            code: u32,
            data: *const CParcel,
            reply: *mut CParcel,
            flags: u32,
        ) -> i32;
    }
}

// Dummy implementation used when running tests.
#[cfg(any(not(target_os = "android"), ndk_build))]
mod cutils {
    // A placeholder of Parcel defined in parcel.cpp
    #[repr(C)]
    pub struct CParcel {
        a: i32,
    }
    #[repr(C)]
    #[derive(Debug)]
    pub struct CNativeHandle {
        pub version: i32,
        pub num_fds: i32,
        pub num_ints: i32, // data[0] size == numFds + numInts
    }

    pub unsafe fn parcel_create() -> *mut CParcel {
        std::ptr::null_mut()
    }
    pub unsafe fn parcel_delete(_p: *mut CParcel) {}
    pub unsafe fn parcel_data_size(_p: *mut CParcel) -> usize {
        0
    }
    pub unsafe fn parcel_set_data_position(_p: *mut CParcel, _pos: usize) {}
    pub unsafe fn parcel_write_interface_token(
        _p: *mut CParcel,
        _iface: *const u8,
        _len: usize,
    ) -> i32 {
        0
    }
    pub unsafe fn parcel_write_int8(_p: *mut CParcel, _v: i8) -> i32 {
        0
    }
    pub unsafe fn parcel_read_int8(_p: *mut CParcel, _v: *mut i8) -> i32 {
        0
    }
    pub unsafe fn parcel_write_uint8(_p: *mut CParcel, _v: u8) -> i32 {
        0
    }
    pub unsafe fn parcel_read_uint8(_p: *mut CParcel, _v: *mut u8) -> i32 {
        0
    }
    pub unsafe fn parcel_write_int16(_p: *mut CParcel, _v: i16) -> i32 {
        0
    }
    pub unsafe fn parcel_read_int16(_p: *mut CParcel, _v: *mut i16) -> i32 {
        0
    }
    pub unsafe fn parcel_write_uint16(_p: *mut CParcel, _v: u16) -> i32 {
        0
    }
    pub unsafe fn parcel_read_uint16(_p: *mut CParcel, _v: &mut u16) -> i32 {
        0
    }
    pub unsafe fn parcel_write_int32(_p: *mut CParcel, _v: i32) -> i32 {
        0
    }
    pub unsafe fn parcel_read_int32(_p: *mut CParcel, _v: *mut i32) -> i32 {
        0
    }
    pub unsafe fn parcel_write_uint32(_p: *mut CParcel, _v: u32) -> i32 {
        0
    }
    pub unsafe fn parcel_read_uint32(_p: *mut CParcel, _v: *mut u32) -> i32 {
        0
    }
    pub unsafe fn parcel_write_int64(_p: *mut CParcel, _v: i64) -> i32 {
        0
    }
    pub unsafe fn parcel_read_int64(_p: *mut CParcel, _v: *mut i64) -> i32 {
        0
    }
    pub unsafe fn parcel_write_uint64(_p: *mut CParcel, _v: u64) -> i32 {
        0
    }
    pub unsafe fn parcel_read_uin64(_p: *mut CParcel, _v: *mut u64) -> i32 {
        0
    }
    pub unsafe fn parcel_write_float(_p: *mut CParcel, _v: f32) -> i32 {
        0
    }
    pub unsafe fn parcel_read_float(_p: *mut CParcel, _v: *mut f32) -> i32 {
        0
    }
    pub unsafe fn parcel_write_double(_p: *mut CParcel, _v: f64) -> i32 {
        0
    }
    pub unsafe fn parcel_read_double(_p: *mut CParcel, _v: *mut f64) -> i32 {
        0
    }
    pub unsafe fn parcel_read_buffer(
        _p: *mut CParcel,
        _buffer_size: usize,
        _buffer_handle: *mut usize,
        _buffer: *mut u8,
    ) -> i32 {
        0
    }
    pub unsafe fn parcel_write_buffer(
        _p: *mut CParcel,
        _buffer: *const u8,
        _buffer_size: usize,
        _buffer_handle: *mut usize,
    ) -> i32 {
        0
    }
    pub unsafe fn parcel_write_embedded_buffer(
        _p: *mut CParcel,
        _buffer: *const u8,
        _buffer_size: usize,
        _buffer_handle: *mut usize,
        _parent_handle: usize,
        _parent_offset: usize,
    ) -> i32 {
        0
    }
    pub unsafe fn parcel_read_embedded_buffer(
        _p: *mut CParcel,
        _buffer_size: usize,
        _buffer_handle: *mut usize,
        _parent_handle: usize,
        _parent_offset: usize,
        _buffer: *mut u8,
    ) -> i32 {
        0
    }
    pub unsafe fn parcel_write_native_handle_no_dup(
        _p: *mut CParcel,
        _handle: *const CNativeHandle,
        _embedded: bool,
        _parent_handle: usize,
        _parent_offset: usize,
    ) -> i32 {
        0
    }
    pub unsafe fn parcel_read_nullable_native_handle_no_dup(
        _p: *mut CParcel,
        _handle: *mut *const CNativeHandle,
        _embedded: bool,
        _parent_handle: usize,
        _parent_offset: usize,
    ) -> i32 {
        0
    }
    // A placeholder of IBinderWrapper defined in ibinder.cpp
    #[repr(C)]
    pub struct CIBinder {
        a: i32,
    }

    // Following functions are defined in ibinder.cpp
    pub unsafe fn ibinder_query_ibinder(
        _iface: *const u8,
        _iface_sz: usize,
        _service_name: *const u8,
        _service_name_sz: usize,
    ) -> *mut CIBinder {
        std::ptr::null_mut()
    }
    pub unsafe fn ibinder_delete(_ibinder: *mut CIBinder) {}
    pub unsafe fn ibinder_transact(
        _ibinder: *mut CIBinder,
        _code: u32,
        _data: *const CParcel,
        _reply: *mut CParcel,
        _flags: u32,
    ) -> i32 {
        0
    }
}

pub mod hidl {
    pub struct Parcel {
        parcel: *mut crate::cutils::CParcel,
        managed: Vec<Vec<u8>>,
    }

    impl Default for Parcel {
        fn default() -> Self {
            Self {
                parcel: unsafe { crate::cutils::parcel_create() },
                managed: Vec::<Vec<u8>>::new(),
            }
        }
    }

    impl Parcel {
        pub fn data_size(&mut self) -> usize {
            unsafe { crate::cutils::parcel_data_size(self.parcel) }
        }

        pub fn set_data_position(&mut self, pos: usize) {
            unsafe { crate::cutils::parcel_set_data_position(self.parcel, pos) }
        }

        pub fn write_iface_token(&mut self, iface: &str) -> Result<(), ()> {
            let c_str = std::ffi::CString::new(iface).unwrap();
            if unsafe {
                crate::cutils::parcel_write_interface_token(
                    self.parcel,
                    c_str.as_ptr() as *const u8,
                    iface.len(),
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_i8(&mut self, v: i8) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_int8(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_i8(&mut self, v: &mut i8) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_int8(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn write_u8(&mut self, v: u8) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_uint8(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_u8(&mut self, v: &mut u8) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_uint8(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_i16(&mut self, v: i16) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_int16(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_i16(&mut self, v: &mut i16) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_int16(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn write_u16(&mut self, v: u16) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_uint16(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_u16(&mut self, v: &mut u16) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_uint16(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_i32(&mut self, v: i32) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_int32(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_i32(&mut self, v: &mut i32) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_int32(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn write_u32(&mut self, v: u32) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_uint32(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_u32(&mut self, v: &mut u32) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_uint32(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_i64(&mut self, v: i64) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_int64(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_i64(&mut self, v: &mut i64) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_int64(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn write_u64(&mut self, v: u64) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_uint64(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_u64(&mut self, v: &mut u64) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_uin64(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_f32(&mut self, v: f32) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_float(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_f32(&mut self, v: &mut f32) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_float(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn write_f64(&mut self, v: f64) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_write_double(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_f64(&mut self, v: &mut f64) -> Result<(), ()> {
            if unsafe { crate::cutils::parcel_read_double(self.parcel, v) } == 0 {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_buffer<T>(&mut self, buffer: &T, buffer_handle: &mut usize) -> Result<(), ()> {
            if unsafe {
                crate::cutils::parcel_write_buffer(
                    self.parcel,
                    std::mem::transmute::<&T, *const u8>(buffer),
                    std::mem::size_of::<T>(),
                    buffer_handle,
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_buffer<T>(
            &mut self,
            buffer_handle: &mut usize,
            buffer: &mut T,
        ) -> Result<(), ()> {
            if unsafe {
                crate::cutils::parcel_read_buffer(
                    self.parcel,
                    std::mem::size_of::<T>(),
                    buffer_handle,
                    std::mem::transmute::<&mut T, *mut u8>(buffer),
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_embedded_buffer<T>(
            &mut self,
            buffer: &T,
            buffer_handle: &mut usize,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            if unsafe {
                crate::cutils::parcel_write_embedded_buffer(
                    self.parcel,
                    std::mem::transmute::<&T, *const u8>(buffer),
                    std::mem::size_of::<T>(),
                    buffer_handle,
                    parent_handle,
                    parent_offset,
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }
        pub fn read_embedded_buffer<T>(
            &mut self,
            buffer_handle: &mut usize,
            parent_handle: usize,
            parent_offset: usize,
            buffer: &mut T,
        ) -> Result<(), ()> {
            if unsafe {
                crate::cutils::parcel_read_embedded_buffer(
                    self.parcel,
                    std::mem::size_of::<T>(),
                    buffer_handle,
                    parent_handle,
                    parent_offset,
                    std::mem::transmute::<&mut T, *mut u8>(buffer),
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }

        pub unsafe fn write_embedded_u8p(
            &mut self,
            buffer: *const u8,
            buffer_size: usize,
            buffer_handle: &mut usize,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            if crate::cutils::parcel_write_embedded_buffer(
                self.parcel,
                buffer,
                buffer_size,
                buffer_handle,
                parent_handle,
                parent_offset,
            ) == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }
        pub unsafe fn read_embedded_u8p(
            &mut self,
            buffer_size: usize,
            buffer_handle: &mut usize,
            parent_handle: usize,
            parent_offset: usize,
            buffer: *mut u8,
        ) -> Result<(), ()> {
            if crate::cutils::parcel_read_embedded_buffer(
                self.parcel,
                buffer_size,
                buffer_handle,
                parent_handle,
                parent_offset,
                buffer,
            ) == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn write_handle(&mut self, handle: &HidlHandle) -> Result<(), ()> {
            let nhandle = HidlNativeHandle::from(handle);
            if unsafe {
                crate::cutils::parcel_write_native_handle_no_dup(
                    self.parcel,
                    nhandle.handle,
                    false,
                    0,
                    0,
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }

        fn _read_handle(
            &mut self,
            embedded: bool,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<HidlHandle, ()> {
            let mut nhandle: *const crate::cutils::CNativeHandle = std::ptr::null();
            if unsafe {
                crate::cutils::parcel_read_nullable_native_handle_no_dup(
                    self.parcel,
                    &mut nhandle,
                    embedded,
                    parent_handle,
                    parent_offset,
                )
            } == 0
            {
                unsafe {
                    let data: *const i32 = nhandle.offset(1) as *const i32;
                    let mut fds = Vec::<i32>::with_capacity((*nhandle).num_fds as usize);
                    let mut didx = 0;
                    for _ in 0..(*nhandle).num_fds {
                        fds.push(*data.offset(didx));
                        didx += 1;
                    }
                    let mut ints = Vec::<i32>::with_capacity((*nhandle).num_ints as usize);
                    for _ in 0..(*nhandle).num_ints {
                        ints.push(*data.offset(didx));
                        didx += 1;
                    }
                    Ok(HidlHandle { fds, ints })
                }
            } else {
                Err(())
            }
        }

        pub fn read_handle(&mut self) -> Result<HidlHandle, ()> {
            self._read_handle(false, 0, 0)
        }

        pub fn write_embedded_handle(
            &mut self,
            handle: &HidlHandle,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            let nhandle = HidlNativeHandle::from(handle);
            if unsafe {
                crate::cutils::parcel_write_native_handle_no_dup(
                    self.parcel,
                    nhandle.handle,
                    true,
                    parent_handle,
                    parent_offset,
                )
            } == 0
            {
                Ok(())
            } else {
                Err(())
            }
        }

        pub fn read_embedded_handle(
            &mut self,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<HidlHandle, ()> {
            self._read_handle(true, parent_handle, parent_offset)
        }

        // Allocate memory for objects and bind their life-cycle with a Parcel.
        //
        // Binder does scatter-gather I/O, so we need to make sure
        // the life of related data blocks are consistent with
        // the parcel it has been written to.  Or, transact() would
        // fail for a data corruption.
        pub fn alloc_obj<T>(&mut self) -> *mut T {
            let mut ov = Vec::<T>::with_capacity(1);
            let ptr = ov.as_mut_ptr();
            let u8v = unsafe {
                std::mem::forget(ov);
                Vec::from_raw_parts(ptr as *mut u8, 0, std::mem::size_of::<T>())
            };
            self.managed.push(u8v);
            ptr
        }

        pub fn alloc_u8(&mut self, len: usize) -> *mut u8 {
            let mut u8v = Vec::<u8>::with_capacity(len);
            let ptr = u8v.as_mut_ptr();
            self.managed.push(u8v);
            ptr
        }
    }
    impl Drop for Parcel {
        fn drop(&mut self) {
            unsafe {
                crate::cutils::parcel_delete(self.parcel);
            }
        }
    }

    /*
     * These are helper functions too complicated to be generated by hidl-gen.
     */
    pub trait ParcelHelper {
        fn write_hidl_string(&mut self, buffer_handle: &mut usize, value: &str) -> Result<(), ()>;
        fn read_hidl_string(&mut self, buffer_handle: &mut usize) -> Result<String, ()>;

        fn write_embedded<T: EmbeddedOps<T>>(
            &mut self,
            buffer: &T,
            em_struct: *const T::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()>;
        fn read_embedded<T: EmbeddedOps<T>>(
            &mut self,
            em_struct: *const T::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<T, ()>;
    }
    #[derive(Clone)]
    #[repr(C)]
    pub struct HidlString {
        // For 32bits, only bufaddr1 will be used.
        // For 64bits, bufaddr1 and bufaddr2 are together as a 64bits pointer.
        pub bufaddr1: u32,
        pub bufaddr2: u32, // Only for 64bits
        pub sz: u32,
        pub owns_buffer: bool,
    }
    impl ParcelHelper for Parcel {
        fn write_hidl_string(&mut self, buffer_handle: &mut usize, value: &str) -> Result<(), ()> {
            let value = String::from(value);

            *buffer_handle = 0;

            let hidl_str = self.alloc_obj::<HidlString>();
            value.prepare_embedded(hidl_str).unwrap();
            let mut handle: usize = 0;
            let r = self.write_buffer(unsafe { &*hidl_str }, &mut handle);
            if r.is_err() {
                return Err(());
            }
            let r = self.write_embedded(&value, hidl_str, handle, 0);

            *buffer_handle = handle;

            r
        }

        fn read_hidl_string(&mut self, buffer_handle: &mut usize) -> Result<String, ()> {
            let mut hidl_str = HidlString {
                bufaddr1: 0,
                bufaddr2: 0,
                sz: 0,
                owns_buffer: false,
            };
            let mut handle: usize = 0;
            *buffer_handle = 0;
            let r = self.read_buffer(&mut handle, &mut hidl_str);
            if r.is_err() {
                return Err(());
            }
            let out = self.read_embedded(&hidl_str, handle, 0);
            *buffer_handle = handle;
            out
        }

        fn write_embedded<T: EmbeddedOps<T>>(
            &mut self,
            buffer: &T,
            em_struct: *const T::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            buffer.write_embedded_to(self, em_struct, parent_handle, parent_offset)
        }
        fn read_embedded<T: EmbeddedOps<T>>(
            &mut self,
            em_struct: *const T::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<T, ()> {
            T::read_embedded_from(self, em_struct, parent_handle, parent_offset)
        }
    }

    pub struct IBinder {
        ibinder: *mut crate::cutils::CIBinder,
    }
    impl IBinder {
        pub fn query_service_manager(iface: &str, service_name: &str) -> Option<IBinder> {
            let iface_cstr = std::ffi::CString::new(iface).unwrap();
            let service_name_cstr = std::ffi::CString::new(service_name).unwrap();
            let ibinder = unsafe {
                crate::cutils::ibinder_query_ibinder(
                    iface_cstr.as_ptr() as *const u8,
                    iface.len(),
                    service_name_cstr.as_ptr() as *const u8,
                    service_name.len(),
                )
            };
            if !ibinder.is_null() {
                Some(IBinder { ibinder })
            } else {
                None
            }
        }

        pub fn transact(
            &self,
            code: u32,
            data: &Parcel,
            reply: &mut Parcel,
            flags: u32,
        ) -> Result<(), ()> {
            let r = unsafe {
                crate::cutils::ibinder_transact(
                    self.ibinder,
                    code,
                    data.parcel,
                    reply.parcel,
                    flags,
                )
            };
            if r == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Drop for IBinder {
        fn drop(&mut self) {
            unsafe {
                crate::cutils::ibinder_delete(self.ibinder);
            }
        }
    }

    #[repr(C)]
    pub struct HidlVec {
        pub bufaddr1: u32,
        pub bufaddr2: u32,
        pub sz: u32,
        pub owns_buf: bool,
    }

    // Operators for embedded buffers.
    //
    // Some types have data out of struct that is pointed by a pointer
    // in the struct. The pointed data blocks will be copied to the
    // parcel following the struct. They are embedded buffers.
    //
    // T is the struct type that contains one or more pointers to
    // embedded buffers.
    //
    pub trait EmbeddedOps<T> {
        // The native struct/type used by T
        //
        // This is the struct written to parcels to carry the content
        // of T.  It may contain pointers to embedded buffers.
        //
        type EmStruct;

        fn elms_size(n: usize) -> usize;

        // Does this type have any embedded buffers?
        //
        fn has_embedded() -> bool {
            false
        }

        // Write embedded buffers pointed from EmStruct to a parcel.
        //
        // The type may adjust parent_offset since it has better
        // knowledge than callers.  For example, Vec can be a caller
        // knowing only the begin positions of elements.  The type of
        // elements should adjust the parent_offset that is
        // the begin position passing from callers.
        //
        fn write_embedded_to(
            &self,
            _parcel: &mut Parcel,
            _em_struct: *const Self::EmStruct,
            _parent_handle: usize,
            _parent_offset: usize,
        ) -> Result<(), ()> {
            Err(())
        }

        // Does this type need to be converted from Rust to native?
        //
        fn need_conversion() -> bool {
            false
        }

        // Prepare the content of a buffer of EmStruct going to be
        // written to a parcel.
        //
        // For some types, it's native type is different from the type in Rust.
        // This type is used to convert rust type to native
        // type/EmStruct if need_conversion() returns true.
        //
        fn prepare_embedded(&self, _em_struct: *mut Self::EmStruct) -> Result<(), ()> {
            Err(())
        }

        // Read embedded buffers pointed from EmStruct from a parcel.
        //
        fn read_embedded_from(
            _parcel: &mut Parcel,
            _em_struct: *const Self::EmStruct,
            _parent_handle: usize,
            _parent_offset: usize,
        ) -> Result<T, ()> {
            Err(())
        }
    }

    impl EmbeddedOps<u8> for u8 {
        type EmStruct = u8;
        fn elms_size(n: usize) -> usize {
            n
        }
    }

    impl EmbeddedOps<u16> for u16 {
        type EmStruct = u16;
        fn elms_size(n: usize) -> usize {
            2 * n
        }
    }

    impl EmbeddedOps<u32> for u32 {
        type EmStruct = u32;
        fn elms_size(n: usize) -> usize {
            4 * n
        }
    }

    impl EmbeddedOps<u64> for u64 {
        type EmStruct = u64;
        fn elms_size(n: usize) -> usize {
            8 * n
        }
    }

    impl EmbeddedOps<i8> for i8 {
        type EmStruct = i8;
        fn elms_size(n: usize) -> usize {
            n
        }
    }

    impl EmbeddedOps<i16> for i16 {
        type EmStruct = i16;
        fn elms_size(n: usize) -> usize {
            2 * n
        }
    }

    impl EmbeddedOps<i32> for i32 {
        type EmStruct = i32;
        fn elms_size(n: usize) -> usize {
            4 * n
        }
    }

    impl EmbeddedOps<i64> for i64 {
        type EmStruct = i64;
        fn elms_size(n: usize) -> usize {
            8 * n
        }
    }

    impl EmbeddedOps<f32> for f32 {
        type EmStruct = f32;
        fn elms_size(n: usize) -> usize {
            4 * n
        }
    }

    impl EmbeddedOps<f64> for f64 {
        type EmStruct = f64;
        fn elms_size(n: usize) -> usize {
            8 * n
        }
    }

    impl<T: EmbeddedOps<T>> EmbeddedOps<Vec<T>> for Vec<T> {
        type EmStruct = HidlVec;

        fn elms_size(n: usize) -> usize {
            std::mem::size_of::<HidlVec>() * n
        }

        fn has_embedded() -> bool {
            true
        }

        fn write_embedded_to(
            &self,
            parcel: &mut Parcel,
            em_struct: *const Self::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            // Elements in a vec may need to a conversion before writting to the parcel.
            // They may also contain embedded buffers.
            //
            let mut handle: usize = 0;
            let elements = self.len();
            assert!(unsafe { (*em_struct).sz as usize } == elements);

            if !T::need_conversion() {
                // Write to the parcel without conversion
                unsafe {
                    parcel
                        .write_embedded_u8p(
                            self.as_ptr() as *const u8,
                            T::elms_size(elements),
                            &mut handle,
                            parent_handle,
                            parent_offset,
                        )
                        .unwrap();
                };
                return Ok(());
            }

            // Convert Rust data to native data and write to the parcel.
            let mut child_structs = Vec::<T::EmStruct>::with_capacity(elements);
            unsafe { child_structs.set_len(elements) };
            for i in 0..elements {
                self[i].prepare_embedded(&mut child_structs[i]).unwrap();
            }
            unsafe {
                parcel
                    .write_embedded_u8p(
                        child_structs.as_ptr() as *const u8,
                        T::elms_size(elements),
                        &mut handle,
                        parent_handle,
                        parent_offset,
                    )
                    .unwrap();
            };

            if !T::has_embedded() {
                return Ok(());
            }

            // Write embedded buffers for every elements.
            for i in 0..elements {
                parcel
                    .write_embedded(&self[i], &child_structs[i], handle, T::elms_size(i))
                    .unwrap();
            }

            Ok(())
        }

        fn need_conversion() -> bool {
            true
        }

        fn prepare_embedded(&self, em_struct: *mut Self::EmStruct) -> Result<(), ()> {
            let data = unsafe { &mut *em_struct };
            data.bufaddr1 = 0;
            data.bufaddr2 = 0;
            data.sz = self.len() as u32;
            data.owns_buf = false;
            Ok(())
        }

        fn read_embedded_from(
            parcel: &mut Parcel,
            em_struct: *const Self::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<Vec<T>, ()> {
            let mut handle: usize = 0;
            let elements = unsafe { (*em_struct).sz as usize };

            if !T::has_embedded() {
                // For types have no embedded data
                let mut v = Vec::<T>::with_capacity(elements);
                unsafe {
                    parcel
                        .read_embedded_u8p(
                            T::elms_size(elements),
                            &mut handle,
                            parent_handle,
                            parent_offset,
                            v.as_mut_ptr() as *mut u8,
                        )
                        .unwrap();
                    v.set_len(elements);
                };
                return Ok(Vec::<T>::new());
            }

            let align = 8; // Should be changed according arch.
            let layout =
                std::alloc::Layout::from_size_align(T::elms_size(elements), align).unwrap();
            let mem = unsafe { std::alloc::alloc(layout) };
            unsafe {
                parcel
                    .read_embedded_u8p(
                        T::elms_size(elements),
                        &mut handle,
                        parent_handle,
                        parent_offset,
                        mem,
                    )
                    .unwrap();
            };
            let mut v = Vec::<T>::with_capacity(elements);
            for i in 0..elements {
                let ev = parcel
                    .read_embedded(
                        unsafe { mem.add(T::elms_size(i)) as *const T::EmStruct },
                        handle,
                        T::elms_size(i),
                    )
                    .unwrap();
                v.push(ev);
            }
            unsafe { std::alloc::dealloc(mem, layout) };
            Ok(Vec::<T>::new())
        }
    }

    impl EmbeddedOps<String> for String {
        type EmStruct = HidlString;

        fn elms_size(n: usize) -> usize {
            std::mem::size_of::<HidlString>() * n
        }

        fn has_embedded() -> bool {
            true
        }

        fn need_conversion() -> bool {
            true
        }

        fn prepare_embedded(&self, em_struct: *mut Self::EmStruct) -> Result<(), ()> {
            let data = unsafe { &mut *em_struct };
            data.bufaddr1 = 0;
            data.bufaddr2 = 0;
            data.sz = self.len() as u32;
            data.owns_buffer = false;
            Ok(())
        }

        fn write_embedded_to(
            &self,
            parcel: &mut Parcel,
            em_struct: *const Self::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            assert!(unsafe { (*em_struct).sz as usize } == self.len());
            let raw_str = parcel.alloc_u8(self.len() + 1);
            let mut handle: usize = 0;
            unsafe {
                std::ptr::copy(self.as_ptr(), raw_str, self.len());
                *raw_str.add(self.len()) = 0; // null
                parcel
                    .write_embedded_u8p(
                        raw_str,
                        self.len() + 1,
                        &mut handle,
                        parent_handle,
                        parent_offset,
                    )
                    .unwrap();
            };
            Ok(())
        }

        fn read_embedded_from(
            parcel: &mut Parcel,
            em_struct: *const Self::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<String, ()> {
            let sz = unsafe { (*em_struct).sz };
            let mut buf = String::with_capacity((sz + 1) as usize);
            let mut handle: usize = 0;
            unsafe {
                parcel
                    .read_embedded_u8p(
                        (sz + 1) as usize,
                        &mut handle,
                        parent_handle,
                        parent_offset,
                        buf.as_mut_ptr(),
                    )
                    .unwrap();
            };
            let raw = buf.as_mut_ptr();
            std::mem::forget(buf);
            Ok(unsafe { String::from_raw_parts(raw, sz as usize, (sz + 1) as usize) })
        }
    }

    pub struct HidlHandle {
        pub fds: Vec<i32>,
        pub ints: Vec<i32>,
    }
    #[repr(C)]
    pub struct hidl_handle {
        bufaddr1: u32,
        bufaddr2: u32,
        owns_handle: bool,
    }

    struct HidlNativeHandle {
        handle: *mut crate::cutils::CNativeHandle,
        layout: std::alloc::Layout,
    }

    impl HidlNativeHandle {
        fn from(handle: &HidlHandle) -> HidlNativeHandle {
            let memsz = std::mem::size_of::<HidlHandle>()
                + std::mem::size_of::<i32>() * (handle.fds.len() + handle.ints.len());
            let align = 8;
            let layout = std::alloc::Layout::from_size_align(memsz, align).unwrap();
            let nhandle: &mut crate::cutils::CNativeHandle;
            unsafe {
                let raw = std::alloc::alloc(layout);
                let data_raw = raw.add(std::mem::size_of::<crate::cutils::CNativeHandle>());

                nhandle = &mut *(raw as *mut crate::cutils::CNativeHandle);
                let data = data_raw as *mut i32;
                nhandle.version = std::mem::size_of::<crate::cutils::CNativeHandle>() as i32;
                nhandle.num_fds = handle.fds.len() as i32;
                nhandle.num_ints = handle.ints.len() as i32;
                let mut i: isize = 0;
                for fd in &handle.fds {
                    *data.offset(i) = *fd;
                    i += 1;
                }
                for int in &handle.ints {
                    *data.offset(i) = *int;
                    i += 1;
                }
            }
            HidlNativeHandle {
                handle: nhandle as *mut crate::cutils::CNativeHandle,
                layout,
            }
        }
    }

    impl Drop for HidlNativeHandle {
        fn drop(&mut self) {
            unsafe { std::alloc::dealloc(self.handle as *mut u8, self.layout) }
        }
    }

    impl Default for HidlHandle {
        fn default() -> Self {
            Self {
                fds: Vec::<i32>::new(),
                ints: Vec::<i32>::new(),
            }
        }
    }

    impl EmbeddedOps<HidlHandle> for HidlHandle {
        type EmStruct = hidl_handle;

        fn elms_size(n: usize) -> usize {
            std::mem::size_of::<hidl_handle>() * n
        }

        fn has_embedded() -> bool {
            true
        }

        fn need_conversion() -> bool {
            true
        }

        fn prepare_embedded(&self, em_struct: *mut Self::EmStruct) -> Result<(), ()> {
            let es = unsafe { &mut *em_struct };
            es.bufaddr1 = 0;
            es.bufaddr2 = 0;
            es.owns_handle = false;
            Ok(())
        }
        fn write_embedded_to(
            &self,
            parcel: &mut Parcel,
            _em_struct: *const Self::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<(), ()> {
            parcel.write_embedded_handle(self, parent_handle, parent_offset)
        }
        fn read_embedded_from(
            parcel: &mut Parcel,
            _em_struct: *const Self::EmStruct,
            parent_handle: usize,
            parent_offset: usize,
        ) -> Result<Self, ()> {
            parcel.read_embedded_handle(parent_handle, parent_offset)
        }
    }
}

#[cfg(test)]
mod tests {
    struct FooType {
        dataf: f32,
        data64: u64,
    }

    #[test]
    fn it_works() {
        let _parcel = crate::hidl::Parcel::new();
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn read_buffer() {
        let src = FooType {
            dataf: 39.12,
            data64: 0xdeadbeefdeadbeef,
        };

        let mut parcel = crate::hidl::Parcel::new();
        parcel.write_f32(src.dataf).unwrap();
        parcel.write_u64(src.data64).unwrap();

        parcel.set_data_position(0);

        let mut dst = FooType {
            dataf: 0.0,
            data64: 0,
        };
        let mut handle: usize = 0;
        parcel.read_buffer(&mut handle, &mut dst).unwrap();

        assert_eq!(src.dataf, dst.dataf);
        assert_eq!(src.data64, dst.data64);
    }
}
