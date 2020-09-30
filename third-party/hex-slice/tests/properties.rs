extern crate suppositions;
extern crate hex_slice;
use hex_slice::AsHex;

use suppositions::*;
use suppositions::generators::*;


#[test]
fn should_start_and_end_with_square_brackets() {
    property(vecs(u8s()))
        .check(|data| {
            let s = format!("{:x}", data.as_hex());
            let len = s.len();
            let first = s.get(..1);
            let last = s.get(len-1..);
            first == Some("[") && last == Some("]")
        });
}

#[test]
fn should_group_u8_as_blocks_of_two() {
    property(vecs(u8s()).map(|data| (data.clone(), format!("{:02x}", data.as_hex()))))
        .check(|(_, s)| {
            let len = s.len();
            let body = match s.get(1..len-1) {
                Some(slice) => slice,
                // Should always have a middle.
                None => return false,
            };

            body.split_whitespace().all(|chunk| chunk.len() == 2)
        });
}

#[test]
fn should_group_u32_as_blocks_of_eight_when_specified() {
    property(vecs(u32s()).map(|data| (data.clone(), format!("{:08x}", data.as_hex()))))
        .check(|(_, s)| {
            let len = s.len();
            let body = match s.get(1..len-1) {
                Some(slice) => slice,
                // Should always have a middle.
                None => return false,
            };

            body.split_whitespace().all(|chunk| chunk.len() == 8)
        });
}

#[test]
fn should_be_decodable_as_original_data() {
    property(vecs(u32s()).map(|data| (data.clone(), format!("{:x}", data.as_hex()))))
        .check(|(orig, s)| -> std::result::Result<bool, _> {
            let len = s.len();
            let body = match s.get(1..len-1) {
                Some(slice) => slice,
                // Should always have a middle.
                None => return Ok(false),
            };

            body.split_whitespace()
                .map(|chunk| u32::from_str_radix(chunk, 16))
                .collect::<Result<Vec<u32>, _>>()
                .map(|d| d == orig)
        });
}

#[test]
fn plain_hex_should_group_u8_as_blocks_of_two() {
    property(vecs(u8s()).map(|data| (data.clone(), format!("{:02x}", data.plain_hex(true)))))
        .check(|(_, s)| {
            s.split_whitespace().all(|chunk| chunk.len() == 2)
        });
}

#[test]
fn plain_hex_should_group_u32_as_blocks_of_eight_when_specified() {
    property(vecs(u32s()).map(|data| (data.clone(), format!("{:08x}", data.plain_hex(true)))))
        .check(|(_, s)| {
            s.split_whitespace().all(|chunk| chunk.len() == 8)
        });
}

#[test]
fn plain_hex_should_be_decodable_as_original_data() {
    property(vecs(u32s()).map(|data| (data.clone(), format!("{:x}", data.plain_hex(true)))))
        .check(|(orig, s)| -> std::result::Result<bool, _> {
            s.split_whitespace()
                .map(|chunk| u32::from_str_radix(chunk, 16))
                .collect::<Result<Vec<u32>, _>>()
                .map(|d| d == orig)
        });
}
