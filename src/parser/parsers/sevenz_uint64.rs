use super::SevenZResult;

use nom::error::context;
use nom::number::complete::u8;

/// 7zip uses a weird packed integer format to represent some u64 values.
/// Parse that and convert it to a normal u64 in native endianness.
pub fn sevenz_uint64(input: &[u8]) -> SevenZResult<u64> {
    fn count_leading_ones(b: u8) -> usize {
        let mut num: usize = 0;
        for shift in 0..8 {
            if ((b << shift) & 0b1000_0000) > 0 {
                num += 1;
            } else {
                return num;
            }
        }
        return num;
    }

    let (input, first_byte) = context("sevenz_uint64 read first byte", u8)(input)?;
    let mut val = 0;
    let mut input_mut: &[u8] = input;
    let leading_ones = count_leading_ones(first_byte);
    for i in 0..leading_ones {
        let (input2, next_byte) = context("sevenz_uint64 read following bytes", u8)(input_mut)?;
        input_mut = input2;
        val += (next_byte as u64) << (i * 8);
    }
    val += (((first_byte as u64) & ((1 << (8 - (leading_ones as u64))) - 1)) << (leading_ones * 8))
        as u64;
    return Ok((input_mut, val));
}
/// Like sevenz_uint64, but convert to usize and return an error if the conversion fails.
pub fn sevenz_uint64_as_usize(input: &[u8]) -> SevenZResult<usize> {
    let (input, as_u64) = context("sevenz_uint64_as_usize as_u64", sevenz_uint64)(input)?;
    let as_usize = to_usize_or_err!(as_u64);
    return Ok((input, as_usize));
}
