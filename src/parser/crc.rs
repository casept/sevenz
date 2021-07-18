//! This module exposes the CRC algorithm used by 7zip.

use crc::*;

pub const CRC_32_7Z: Algorithm<u32> = Algorithm {
    poly: 0x04c11db7,
    init: 0xffffffff,
    refin: true,
    refout: true,
    xorout: 0xffffffff,
    check: 0xfc891918,
    residue: 0xc704dd7b,
};

pub fn sevenz_crc(input: &[u8]) -> u32 {
    let algo = Crc::<u32>::new(&CRC_32_7Z);
    let mut digest = algo.digest();
    digest.update(input);
    return digest.finalize();
}
