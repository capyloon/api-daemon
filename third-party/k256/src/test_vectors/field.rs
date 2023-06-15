//! Test vectors for the secp256k1 base field.

use hex_literal::hex;

/// Repeated doubling of the multiplicative identity.
pub const DBL_TEST_VECTORS: &[[u8; 32]] = &[
    hex!("0000000000000000000000000000000000000000000000000000000000000001"),
    hex!("0000000000000000000000000000000000000000000000000000000000000002"),
    hex!("0000000000000000000000000000000000000000000000000000000000000004"),
    hex!("0000000000000000000000000000000000000000000000000000000000000008"),
    hex!("0000000000000000000000000000000000000000000000000000000000000010"),
    hex!("0000000000000000000000000000000000000000000000000000000000000020"),
    hex!("0000000000000000000000000000000000000000000000000000000000000040"),
    hex!("0000000000000000000000000000000000000000000000000000000000000080"),
    hex!("0000000000000000000000000000000000000000000000000000000000000100"),
    hex!("0000000000000000000000000000000000000000000000000000000000000200"),
    hex!("0000000000000000000000000000000000000000000000000000000000000400"),
    hex!("0000000000000000000000000000000000000000000000000000000000000800"),
    hex!("0000000000000000000000000000000000000000000000000000000000001000"),
    hex!("0000000000000000000000000000000000000000000000000000000000002000"),
    hex!("0000000000000000000000000000000000000000000000000000000000004000"),
    hex!("0000000000000000000000000000000000000000000000000000000000008000"),
    hex!("0000000000000000000000000000000000000000000000000000000000010000"),
    hex!("0000000000000000000000000000000000000000000000000000000000020000"),
    hex!("0000000000000000000000000000000000000000000000000000000000040000"),
    hex!("0000000000000000000000000000000000000000000000000000000000080000"),
    hex!("0000000000000000000000000000000000000000000000000000000000100000"),
    hex!("0000000000000000000000000000000000000000000000000000000000200000"),
    hex!("0000000000000000000000000000000000000000000000000000000000400000"),
    hex!("0000000000000000000000000000000000000000000000000000000000800000"),
    hex!("0000000000000000000000000000000000000000000000000000000001000000"),
    hex!("0000000000000000000000000000000000000000000000000000000002000000"),
    hex!("0000000000000000000000000000000000000000000000000000000004000000"),
    hex!("0000000000000000000000000000000000000000000000000000000008000000"),
    hex!("0000000000000000000000000000000000000000000000000000000010000000"),
    hex!("0000000000000000000000000000000000000000000000000000000020000000"),
    hex!("0000000000000000000000000000000000000000000000000000000040000000"),
    hex!("0000000000000000000000000000000000000000000000000000000080000000"),
    hex!("0000000000000000000000000000000000000000000000000000000100000000"),
    hex!("0000000000000000000000000000000000000000000000000000000200000000"),
    hex!("0000000000000000000000000000000000000000000000000000000400000000"),
    hex!("0000000000000000000000000000000000000000000000000000000800000000"),
    hex!("0000000000000000000000000000000000000000000000000000001000000000"),
    hex!("0000000000000000000000000000000000000000000000000000002000000000"),
    hex!("0000000000000000000000000000000000000000000000000000004000000000"),
    hex!("0000000000000000000000000000000000000000000000000000008000000000"),
    hex!("0000000000000000000000000000000000000000000000000000010000000000"),
    hex!("0000000000000000000000000000000000000000000000000000020000000000"),
    hex!("0000000000000000000000000000000000000000000000000000040000000000"),
    hex!("0000000000000000000000000000000000000000000000000000080000000000"),
    hex!("0000000000000000000000000000000000000000000000000000100000000000"),
    hex!("0000000000000000000000000000000000000000000000000000200000000000"),
    hex!("0000000000000000000000000000000000000000000000000000400000000000"),
    hex!("0000000000000000000000000000000000000000000000000000800000000000"),
    hex!("0000000000000000000000000000000000000000000000000001000000000000"),
    hex!("0000000000000000000000000000000000000000000000000002000000000000"),
    hex!("0000000000000000000000000000000000000000000000000004000000000000"),
    hex!("0000000000000000000000000000000000000000000000000008000000000000"),
    hex!("0000000000000000000000000000000000000000000000000010000000000000"),
    hex!("0000000000000000000000000000000000000000000000000020000000000000"),
    hex!("0000000000000000000000000000000000000000000000000040000000000000"),
    hex!("0000000000000000000000000000000000000000000000000080000000000000"),
    hex!("0000000000000000000000000000000000000000000000000100000000000000"),
    hex!("0000000000000000000000000000000000000000000000000200000000000000"),
    hex!("0000000000000000000000000000000000000000000000000400000000000000"),
    hex!("0000000000000000000000000000000000000000000000000800000000000000"),
    hex!("0000000000000000000000000000000000000000000000001000000000000000"),
    hex!("0000000000000000000000000000000000000000000000002000000000000000"),
    hex!("0000000000000000000000000000000000000000000000004000000000000000"),
    hex!("0000000000000000000000000000000000000000000000008000000000000000"),
    hex!("0000000000000000000000000000000000000000000000010000000000000000"),
    hex!("0000000000000000000000000000000000000000000000020000000000000000"),
    hex!("0000000000000000000000000000000000000000000000040000000000000000"),
    hex!("0000000000000000000000000000000000000000000000080000000000000000"),
    hex!("0000000000000000000000000000000000000000000000100000000000000000"),
    hex!("0000000000000000000000000000000000000000000000200000000000000000"),
    hex!("0000000000000000000000000000000000000000000000400000000000000000"),
    hex!("0000000000000000000000000000000000000000000000800000000000000000"),
    hex!("0000000000000000000000000000000000000000000001000000000000000000"),
    hex!("0000000000000000000000000000000000000000000002000000000000000000"),
    hex!("0000000000000000000000000000000000000000000004000000000000000000"),
    hex!("0000000000000000000000000000000000000000000008000000000000000000"),
    hex!("0000000000000000000000000000000000000000000010000000000000000000"),
    hex!("0000000000000000000000000000000000000000000020000000000000000000"),
    hex!("0000000000000000000000000000000000000000000040000000000000000000"),
    hex!("0000000000000000000000000000000000000000000080000000000000000000"),
    hex!("0000000000000000000000000000000000000000000100000000000000000000"),
    hex!("0000000000000000000000000000000000000000000200000000000000000000"),
    hex!("0000000000000000000000000000000000000000000400000000000000000000"),
    hex!("0000000000000000000000000000000000000000000800000000000000000000"),
    hex!("0000000000000000000000000000000000000000001000000000000000000000"),
    hex!("0000000000000000000000000000000000000000002000000000000000000000"),
    hex!("0000000000000000000000000000000000000000004000000000000000000000"),
    hex!("0000000000000000000000000000000000000000008000000000000000000000"),
    hex!("0000000000000000000000000000000000000000010000000000000000000000"),
    hex!("0000000000000000000000000000000000000000020000000000000000000000"),
    hex!("0000000000000000000000000000000000000000040000000000000000000000"),
    hex!("0000000000000000000000000000000000000000080000000000000000000000"),
    hex!("0000000000000000000000000000000000000000100000000000000000000000"),
    hex!("0000000000000000000000000000000000000000200000000000000000000000"),
    hex!("0000000000000000000000000000000000000000400000000000000000000000"),
    hex!("0000000000000000000000000000000000000000800000000000000000000000"),
    hex!("0000000000000000000000000000000000000001000000000000000000000000"),
    hex!("0000000000000000000000000000000000000002000000000000000000000000"),
    hex!("0000000000000000000000000000000000000004000000000000000000000000"),
    hex!("0000000000000000000000000000000000000008000000000000000000000000"),
    hex!("0000000000000000000000000000000000000010000000000000000000000000"),
    hex!("0000000000000000000000000000000000000020000000000000000000000000"),
    hex!("0000000000000000000000000000000000000040000000000000000000000000"),
    hex!("0000000000000000000000000000000000000080000000000000000000000000"),
    hex!("0000000000000000000000000000000000000100000000000000000000000000"),
    hex!("0000000000000000000000000000000000000200000000000000000000000000"),
    hex!("0000000000000000000000000000000000000400000000000000000000000000"),
    hex!("0000000000000000000000000000000000000800000000000000000000000000"),
    hex!("0000000000000000000000000000000000001000000000000000000000000000"),
    hex!("0000000000000000000000000000000000002000000000000000000000000000"),
    hex!("0000000000000000000000000000000000004000000000000000000000000000"),
    hex!("0000000000000000000000000000000000008000000000000000000000000000"),
    hex!("0000000000000000000000000000000000010000000000000000000000000000"),
    hex!("0000000000000000000000000000000000020000000000000000000000000000"),
    hex!("0000000000000000000000000000000000040000000000000000000000000000"),
    hex!("0000000000000000000000000000000000080000000000000000000000000000"),
    hex!("0000000000000000000000000000000000100000000000000000000000000000"),
    hex!("0000000000000000000000000000000000200000000000000000000000000000"),
    hex!("0000000000000000000000000000000000400000000000000000000000000000"),
    hex!("0000000000000000000000000000000000800000000000000000000000000000"),
    hex!("0000000000000000000000000000000001000000000000000000000000000000"),
    hex!("0000000000000000000000000000000002000000000000000000000000000000"),
    hex!("0000000000000000000000000000000004000000000000000000000000000000"),
    hex!("0000000000000000000000000000000008000000000000000000000000000000"),
    hex!("0000000000000000000000000000000010000000000000000000000000000000"),
    hex!("0000000000000000000000000000000020000000000000000000000000000000"),
    hex!("0000000000000000000000000000000040000000000000000000000000000000"),
    hex!("0000000000000000000000000000000080000000000000000000000000000000"),
    hex!("0000000000000000000000000000000100000000000000000000000000000000"),
    hex!("0000000000000000000000000000000200000000000000000000000000000000"),
    hex!("0000000000000000000000000000000400000000000000000000000000000000"),
    hex!("0000000000000000000000000000000800000000000000000000000000000000"),
    hex!("0000000000000000000000000000001000000000000000000000000000000000"),
    hex!("0000000000000000000000000000002000000000000000000000000000000000"),
    hex!("0000000000000000000000000000004000000000000000000000000000000000"),
    hex!("0000000000000000000000000000008000000000000000000000000000000000"),
    hex!("0000000000000000000000000000010000000000000000000000000000000000"),
    hex!("0000000000000000000000000000020000000000000000000000000000000000"),
    hex!("0000000000000000000000000000040000000000000000000000000000000000"),
    hex!("0000000000000000000000000000080000000000000000000000000000000000"),
    hex!("0000000000000000000000000000100000000000000000000000000000000000"),
    hex!("0000000000000000000000000000200000000000000000000000000000000000"),
    hex!("0000000000000000000000000000400000000000000000000000000000000000"),
    hex!("0000000000000000000000000000800000000000000000000000000000000000"),
    hex!("0000000000000000000000000001000000000000000000000000000000000000"),
    hex!("0000000000000000000000000002000000000000000000000000000000000000"),
    hex!("0000000000000000000000000004000000000000000000000000000000000000"),
    hex!("0000000000000000000000000008000000000000000000000000000000000000"),
    hex!("0000000000000000000000000010000000000000000000000000000000000000"),
    hex!("0000000000000000000000000020000000000000000000000000000000000000"),
    hex!("0000000000000000000000000040000000000000000000000000000000000000"),
    hex!("0000000000000000000000000080000000000000000000000000000000000000"),
    hex!("0000000000000000000000000100000000000000000000000000000000000000"),
    hex!("0000000000000000000000000200000000000000000000000000000000000000"),
    hex!("0000000000000000000000000400000000000000000000000000000000000000"),
    hex!("0000000000000000000000000800000000000000000000000000000000000000"),
    hex!("0000000000000000000000001000000000000000000000000000000000000000"),
    hex!("0000000000000000000000002000000000000000000000000000000000000000"),
    hex!("0000000000000000000000004000000000000000000000000000000000000000"),
    hex!("0000000000000000000000008000000000000000000000000000000000000000"),
    hex!("0000000000000000000000010000000000000000000000000000000000000000"),
    hex!("0000000000000000000000020000000000000000000000000000000000000000"),
    hex!("0000000000000000000000040000000000000000000000000000000000000000"),
    hex!("0000000000000000000000080000000000000000000000000000000000000000"),
    hex!("0000000000000000000000100000000000000000000000000000000000000000"),
    hex!("0000000000000000000000200000000000000000000000000000000000000000"),
    hex!("0000000000000000000000400000000000000000000000000000000000000000"),
    hex!("0000000000000000000000800000000000000000000000000000000000000000"),
    hex!("0000000000000000000001000000000000000000000000000000000000000000"),
    hex!("0000000000000000000002000000000000000000000000000000000000000000"),
    hex!("0000000000000000000004000000000000000000000000000000000000000000"),
    hex!("0000000000000000000008000000000000000000000000000000000000000000"),
    hex!("0000000000000000000010000000000000000000000000000000000000000000"),
    hex!("0000000000000000000020000000000000000000000000000000000000000000"),
    hex!("0000000000000000000040000000000000000000000000000000000000000000"),
    hex!("0000000000000000000080000000000000000000000000000000000000000000"),
    hex!("0000000000000000000100000000000000000000000000000000000000000000"),
    hex!("0000000000000000000200000000000000000000000000000000000000000000"),
    hex!("0000000000000000000400000000000000000000000000000000000000000000"),
    hex!("0000000000000000000800000000000000000000000000000000000000000000"),
    hex!("0000000000000000001000000000000000000000000000000000000000000000"),
    hex!("0000000000000000002000000000000000000000000000000000000000000000"),
    hex!("0000000000000000004000000000000000000000000000000000000000000000"),
    hex!("0000000000000000008000000000000000000000000000000000000000000000"),
    hex!("0000000000000000010000000000000000000000000000000000000000000000"),
    hex!("0000000000000000020000000000000000000000000000000000000000000000"),
    hex!("0000000000000000040000000000000000000000000000000000000000000000"),
    hex!("0000000000000000080000000000000000000000000000000000000000000000"),
    hex!("0000000000000000100000000000000000000000000000000000000000000000"),
    hex!("0000000000000000200000000000000000000000000000000000000000000000"),
    hex!("0000000000000000400000000000000000000000000000000000000000000000"),
    hex!("0000000000000000800000000000000000000000000000000000000000000000"),
    hex!("0000000000000001000000000000000000000000000000000000000000000000"),
    hex!("0000000000000002000000000000000000000000000000000000000000000000"),
    hex!("0000000000000004000000000000000000000000000000000000000000000000"),
    hex!("0000000000000008000000000000000000000000000000000000000000000000"),
    hex!("0000000000000010000000000000000000000000000000000000000000000000"),
    hex!("0000000000000020000000000000000000000000000000000000000000000000"),
    hex!("0000000000000040000000000000000000000000000000000000000000000000"),
    hex!("0000000000000080000000000000000000000000000000000000000000000000"),
    hex!("0000000000000100000000000000000000000000000000000000000000000000"),
    hex!("0000000000000200000000000000000000000000000000000000000000000000"),
    hex!("0000000000000400000000000000000000000000000000000000000000000000"),
    hex!("0000000000000800000000000000000000000000000000000000000000000000"),
    hex!("0000000000001000000000000000000000000000000000000000000000000000"),
    hex!("0000000000002000000000000000000000000000000000000000000000000000"),
    hex!("0000000000004000000000000000000000000000000000000000000000000000"),
    hex!("0000000000008000000000000000000000000000000000000000000000000000"),
    hex!("0000000000010000000000000000000000000000000000000000000000000000"),
    hex!("0000000000020000000000000000000000000000000000000000000000000000"),
    hex!("0000000000040000000000000000000000000000000000000000000000000000"),
    hex!("0000000000080000000000000000000000000000000000000000000000000000"),
    hex!("0000000000100000000000000000000000000000000000000000000000000000"),
    hex!("0000000000200000000000000000000000000000000000000000000000000000"),
    hex!("0000000000400000000000000000000000000000000000000000000000000000"),
    hex!("0000000000800000000000000000000000000000000000000000000000000000"),
    hex!("0000000001000000000000000000000000000000000000000000000000000000"),
    hex!("0000000002000000000000000000000000000000000000000000000000000000"),
    hex!("0000000004000000000000000000000000000000000000000000000000000000"),
    hex!("0000000008000000000000000000000000000000000000000000000000000000"),
    hex!("0000000010000000000000000000000000000000000000000000000000000000"),
    hex!("0000000020000000000000000000000000000000000000000000000000000000"),
    hex!("0000000040000000000000000000000000000000000000000000000000000000"),
    hex!("0000000080000000000000000000000000000000000000000000000000000000"),
    hex!("0000000100000000000000000000000000000000000000000000000000000000"),
    hex!("0000000200000000000000000000000000000000000000000000000000000000"),
    hex!("0000000400000000000000000000000000000000000000000000000000000000"),
    hex!("0000000800000000000000000000000000000000000000000000000000000000"),
    hex!("0000001000000000000000000000000000000000000000000000000000000000"),
    hex!("0000002000000000000000000000000000000000000000000000000000000000"),
    hex!("0000004000000000000000000000000000000000000000000000000000000000"),
    hex!("0000008000000000000000000000000000000000000000000000000000000000"),
    hex!("0000010000000000000000000000000000000000000000000000000000000000"),
    hex!("0000020000000000000000000000000000000000000000000000000000000000"),
    hex!("0000040000000000000000000000000000000000000000000000000000000000"),
    hex!("0000080000000000000000000000000000000000000000000000000000000000"),
    hex!("0000100000000000000000000000000000000000000000000000000000000000"),
    hex!("0000200000000000000000000000000000000000000000000000000000000000"),
    hex!("0000400000000000000000000000000000000000000000000000000000000000"),
    hex!("0000800000000000000000000000000000000000000000000000000000000000"),
    hex!("0001000000000000000000000000000000000000000000000000000000000000"),
    hex!("0002000000000000000000000000000000000000000000000000000000000000"),
    hex!("0004000000000000000000000000000000000000000000000000000000000000"),
    hex!("0008000000000000000000000000000000000000000000000000000000000000"),
    hex!("0010000000000000000000000000000000000000000000000000000000000000"),
    hex!("0020000000000000000000000000000000000000000000000000000000000000"),
    hex!("0040000000000000000000000000000000000000000000000000000000000000"),
    hex!("0080000000000000000000000000000000000000000000000000000000000000"),
    hex!("0100000000000000000000000000000000000000000000000000000000000000"),
    hex!("0200000000000000000000000000000000000000000000000000000000000000"),
];
