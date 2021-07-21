use super::SevenZResult;

extern crate std;

use alloc::vec;
use alloc::vec::Vec;
use core::cmp::min;
use core::iter::FromIterator;

use bitvec::prelude::*;
use nom::error::context;

/// Extracts the given number of bits from a byte input into a `BitVec`, dropping any leftover bits from the last byte.
pub fn take_bitvec(input: &[u8], num_bits: usize) -> SevenZResult<BitVec> {
    let num_tail_bits = num_bits % 8;

    // There's no way I can tell to read into a bitvec directly,
    // so this gross workaround of reading into several usizes is needed
    let mut left = num_bits;
    let mut input_mut = input;
    let mut raw_bits: Vec<u8> = vec![];
    while left > 0 {
        let (input, some_bits): (&[u8], u8) = match context::<_, nom::error::Error<&[u8]>, _, _>(
            "take_bitvec some_bits",
            nom::bits::bits::<_, _, nom::error::Error<(&[u8], usize)>, _, _>(
                nom::bits::complete::take::<_, _, _, _>(min(left, 8)),
            ),
        )(input_mut)
        {
            Err(e) => panic!("TODO: Cleanup"),
            Ok(val) => val,
        };
        std::println!("Read {:>8b}", some_bits);
        raw_bits.push(some_bits);
        input_mut = input;

        if left >= 8 {
            left = left - 8;
        } else if left == num_tail_bits {
            left = left - num_tail_bits;
        } else {
            panic!("I'm not supposed to be reached, this is a bug");
        }
    }
    let input = input_mut;

    // Do the conversion
    let mut bv: BitVec = BitVec::from_iter(raw_bits.view_bits::<Lsb0>());
    std::println!("Before trim: {}", bv);
    // Remove excess
    let excess_bits = if num_tail_bits != 0 {
        8 - num_tail_bits
    } else {
        0
    };
    for _ in 0..excess_bits {
        bv.pop();
    }
    std::println!("After trim: {}", bv);
    return Ok((input, bv));
}
