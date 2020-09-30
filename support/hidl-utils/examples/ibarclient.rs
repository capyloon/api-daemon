/* -*- Mode: rust; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* vim: set ts=8 sts=4 et sw=4 tw=80: */
/*
 * This client should be run with |kaios.test.bar@1.0-service| that
 * provides a default instance of IBar over binder.
 */
use hidl_utils::hidl;
use hidl_utils::hidl::EmbeddedOps;
use hidl_utils::hidl::ParcelHelper;
use std::io::Read;
use std::os::unix::io::AsRawFd;

const IFACE: &str = "kaios.test.bar@1.0::IBar";
const SERVICE: &str = "default";

#[repr(C)]
struct Position {
    x: i32,
    y: i32,
    msg1: String,
    msg2: String,
}
#[repr(C)]
struct Position_embedded {
    x: i32,
    y: i32,
    msg1: <String as hidl::EmbeddedOps<String>>::EM_STRUCT,
    msg2: <String as hidl::EmbeddedOps<String>>::EM_STRUCT,
}

impl EmbeddedOps<Position> for Position {
    type EM_STRUCT = Position_embedded;
    fn elms_size(n: usize) -> usize {
        n * std::mem::size_of::<Self>()
    }
    fn has_embedded() -> bool {
        true
    }
    fn need_conversion() -> bool {
        true
    }
    fn write_embedded_to(
        &self,
        parcel: &mut hidl::Parcel,
        em_struct: *const Self::EM_STRUCT,
        parent_handle: usize,
        parent_offset: usize,
    ) -> Result<(), ()> {
        unsafe {
            parcel
                .write_embedded(
                    &self.msg1,
                    &(*em_struct).msg1,
                    parent_handle,
                    parent_offset + 8,
                )
                .unwrap();
            parcel
                .write_embedded(
                    &self.msg2,
                    &(*em_struct).msg2,
                    parent_handle,
                    parent_offset + 24,
                )
                .unwrap();
        }
        Ok(())
    }
    fn read_embedded_from(
        parcel: &mut hidl::Parcel,
        em_struct: *const Self::EM_STRUCT,
        parent_handle: usize,
        parent_offset: usize,
    ) -> Result<(Self), ()> {
        unsafe {
            let ret = Position {
                x: (*em_struct).x,
                y: (*em_struct).y,
                msg1: {
                    let _v = parcel
                        .read_embedded(&(*em_struct).msg1, parent_handle, parent_offset + 8)
                        .unwrap();
                    _v
                },
                msg2: {
                    let _v = parcel
                        .read_embedded(&(*em_struct).msg2, parent_handle, parent_offset + 24)
                        .unwrap();
                    _v
                },
            };
            Ok(ret)
        }
    }
    fn prepare_embedded(&self, em_struct: *mut Self::EM_STRUCT) -> Result<(), ()> {
        unsafe {
            (*em_struct).x = self.x;
            (*em_struct).y = self.y;
            self.msg1.prepare_embedded(&mut (*em_struct).msg1).unwrap();
            self.msg2.prepare_embedded(&mut (*em_struct).msg2).unwrap();
        };
        Ok(())
    }
}

fn get_position() {
    println!("get_position");
    let bar = hidl::IBinder::query_service_manager(IFACE, SERVICE).unwrap();
    let mut parcel = hidl::Parcel::new();
    parcel.write_iface_token(IFACE).unwrap();
    parcel.write_i32(11).unwrap();

    let pos = Position {
        x: 0,
        y: 0,
        msg1: "POS1".to_string(),
        msg2: "POS2".to_string(),
    };
    let _v = &pos;
    let mut handle: usize = 0;
    let _em_struct = parcel.alloc_obj::<<Position as EmbeddedOps<Position>>::EM_STRUCT>();
    _v.prepare_embedded(_em_struct).unwrap();
    {
        let _v = unsafe { &*_em_struct };
        parcel.write_buffer(_v, &mut handle).unwrap();
    };
    parcel.write_embedded(_v, _em_struct, handle, 0).unwrap();

    let mut reply = hidl::Parcel::new();
    bar.transact(1, &parcel, &mut reply, 0).unwrap();

    let mut version: u32 = 0;
    reply.read_u32(&mut version).unwrap();
    let mut handle: usize = 0;
    let mut _vv = Vec::<Position_embedded>::with_capacity(1);
    let _v = unsafe { &mut *_vv.as_mut_ptr() };
    reply.read_buffer(&mut handle, _v).unwrap();
    let pos: Position = reply.read_embedded(_v, handle, 0).unwrap();

    println!(
        "  Result: version={} x={}, y={}, msg1={}, msg2={}",
        version, pos.x, pos.y, pos.msg1, pos.msg2
    );
    if pos.x != 211 || pos.y != 111 {
        println!("Error: it should be (211, 111)!");
    }
}

fn send_msg() {
    println!("send_msg");
    let bar = hidl::IBinder::query_service_manager(IFACE, SERVICE).unwrap();
    let mut parcel = hidl::Parcel::new();
    parcel.write_iface_token(IFACE).unwrap();
    let mut _handle: usize = 0;
    parcel.write_hidl_string(&mut _handle, "Hello!!").unwrap();

    let mut reply = hidl::Parcel::new();
    bar.transact(3, &parcel, &mut reply, 0).unwrap();

    let mut version: u32 = 0;
    reply.read_u32(&mut version).unwrap();

    let mut handle: usize = 0;
    let msg = reply.read_hidl_string(&mut handle).unwrap();

    println!("  Msg: {}", msg);
    assert!(msg == "from server!");
}

fn send_handle() {
    println!("send_handle");
    let (mut sock1, sock2) = std::os::unix::net::UnixStream::pair().unwrap();
    let fd2 = sock2.as_raw_fd();
    let mut handle = hidl_utils::hidl::HidlHandle::new();
    handle.fds.push(fd2);

    let bar = hidl::IBinder::query_service_manager(IFACE, SERVICE).unwrap();
    let mut parcel = hidl::Parcel::new();
    parcel.write_iface_token(IFACE).unwrap();
    parcel.write_handle(&handle).unwrap();

    let mut reply = hidl::Parcel::new();
    bar.transact(4, &parcel, &mut reply, 0).unwrap();

    // Force sock2 closing.
    // You should be very careful to manage life-cycle of FDs.
    // They should be alive at least when calling transact().
    drop(sock2);

    let mut version: u32 = 0;
    reply.read_u32(&mut version).unwrap();

    let mut msg = String::new();
    sock1.read_to_string(&mut msg).unwrap();
    println!("  Msg: {}", msg);
    assert!(msg == "HELLO World");
}

fn main() {
    get_position();
    send_msg();
    send_handle();
}
