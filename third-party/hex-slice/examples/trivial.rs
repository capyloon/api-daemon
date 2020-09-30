extern crate hex_slice;
use hex_slice::AsHex;

fn main() {
    let foo = vec![0u32, 1 ,2 ,3];
    println!("{:x}", foo.as_hex());
}

