use super::SevenZResult;

use alloc::vec;
use alloc::vec::Vec;
use core::cmp::min;

use bitvec::prelude::*;
use nom::bits::bits;
use nom::bits::complete::take;
use nom::error::context;

/// Extracts the given number of bits from a byte input into a `BitVec`, dropping any leftover bits from the last byte.
pub fn take_bitvec(input: &[u8], num_bits: usize) -> SevenZResult<BitVec> {
    // There's no way I can tell to read into a bitvec directly,
    // so this gross workaround of reading into several usizes is needed
    let usize_size = core::mem::size_of::<usize>() * 8;
    let num_tail_bits = num_bits % usize_size;
    let mut left = num_bits;
    let mut input_mut = input;
    let mut raw_bits: Vec<usize> = vec![];
    while left > 0 {
        let (input, some_bits): (&[u8], usize) = match context::<_, nom::error::Error<&[u8]>, _, _>(
            "take_bitvec some_bits",
            bits::<_, _, nom::error::Error<(&[u8], usize)>, _, _>(take(min(left, usize_size))),
        )(input_mut)
        {
            Err(e) => panic!("TODO: Cleanup"),
            Ok(val) => val,
        };
        raw_bits.push(some_bits);
        input_mut = input;

        if left >= usize_size {
            left = left - usize_size;
        } else if left == num_tail_bits {
            left = 0;
        } else {
            panic!("I'm not supposed to be reached, this is a bug");
        }
    }
    let input = input_mut;

    // Do the conversion
    let mut bv: BitVec = BitVec::from_vec(raw_bits);
    // Remove excess
    let excess_bits = if num_tail_bits != 0 {
        usize_size - num_tail_bits
    } else {
        0
    };
    if excess_bits > 0 {
        for _ in 0..excess_bits {
            bv.pop();
        }
    }

    // This is probably needed because the vector is converted LSB first,
    // whereas nom pushes bits MSB first.
    // I could probably tell BitVec to generate an MSB BitVec, but I'm not sure how to
    // convert that to the standard LSB BitVec.
    bv.as_mut_bitslice()
        .chunks_mut(usize_size)
        .for_each(|x| x.reverse());
    return Ok((input, bv));
}
