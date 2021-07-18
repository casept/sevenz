//! Custom nom combinators for the 7z format

use super::crc;
use super::types::*;

use alloc::vec;
use alloc::vec::*;
use core::convert::*;

use nom::bytes::complete::{tag, take};
use nom::error::*;
use nom::number::complete::{u32, u64, u8};
use nom::number::Endianness;
use nom::IResult;

/// Header magic bytes
const MAGIC: [u8; 6] = [b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

/// 7zip uses a weird packed integer format to represent some u64 values.
/// Parse that and convert it to a normal u64 in native endianness.
pub fn sevenz_uint64(input: &[u8]) -> IResult<&[u8], u64> {
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

    let (input, first_byte) = u8(input)?;
    let mut val = 0;
    let mut input_mut: &[u8] = input;
    let leading_ones = count_leading_ones(first_byte);
    for i in 0..leading_ones {
        let (input2, next_byte) = u8(input_mut)?;
        input_mut = input2;
        val += (next_byte as u64) << (i * 8);
    }
    val += (((first_byte as u64) & ((1 << (8 - (leading_ones as u64))) - 1)) << (leading_ones * 8))
        as u64;
    return Ok((input_mut, val));
}

/// The errors that may be returned by the parser.
#[derive(Debug, PartialEq)]
pub enum SevenZParserError<I> {
    Nom(I, nom::error::ErrorKind),
    // Crc(expected, got)
    Crc(u32, u32),
    // InvalidPropertyID(id)
    InvalidPropertyID(u8),
    ToUsizeConversionFailure(<usize as TryFrom<u64>>::Error),
}

impl<I> ParseError<I> for SevenZParserError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        SevenZParserError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

// This should be a From<...> implementation, but that doesn't compile.
impl<I> SevenZParserError<I> {
    fn from_err(e: nom::Err<nom::error::Error<I>>) -> Self {
        use nom::Err::*;
        match e {
            Incomplete(_) => panic!("Encountered incomplete, not sure what to do"),
            Error(inner) => return SevenZParserError::from_error_kind(inner.input, inner.code),
            Failure(inner) => return SevenZParserError::from_error_kind(inner.input, inner.code),
        }
    }
}

/// Macro for converting from the error emitted by builtin combinators to our error type.
macro_rules! to_err {
( $( $x:expr ),+ ) => {
        {
            $(
                match $x {
        Ok(res) => res,
        Err(e) => return Err(nom::Err::Error(SevenZParserError::from_err(e))),
    }
            )+
        }
    };
}

/// Macro for converting form u64 to usize, or returning the correct error if conversion not possible
macro_rules! to_usize_or_err {
( $( $x:expr ),+ ) => {
        {
            $(
                match usize::try_from($x) {
        Ok(res) => res,
        Err(e) => return Err(nom::Err::Error(SevenZParserError::ToUsizeConversionFailure(e))),
    }
            )+
        }
    };
}

pub fn archive_version(input: &[u8]) -> IResult<&[u8], ArchiveVersion> {
    let (input, major) = u8(input)?;
    let (input, minor) = u8(input)?;
    return Ok((input, ArchiveVersion { major, minor }));
}

pub fn start_header(input: &[u8]) -> IResult<&[u8], StartHeader> {
    let (input, next_header_offset) = u64(Endianness::Little)(input)?;
    let (input, next_header_size) = u64(Endianness::Little)(input)?;
    let (input, next_header_crc) = u32(Endianness::Little)(input)?;
    return Ok((
        input,
        StartHeader {
            next_header_offset,
            next_header_size,
            next_header_crc,
        },
    ));
}

pub fn signature_header(input: &[u8]) -> IResult<&[u8], SignatureHeader, SevenZParserError<&[u8]>> {
    let (input, _) = tag(MAGIC)(input)?;
    // This is super ugly, but not sure how to solve this more elegantly
    let (input, archive_version) = to_err!(archive_version(input));
    let (input, start_header_crc) = u32(Endianness::Little)(input)?;
    let (_, raw_start_header) = take(START_HEADER_SIZE_BYTES)(input)?;
    let (input, start_header) = to_err!(start_header(input));

    // Verify CRC
    let calculated_crc = crc::sevenz_crc(raw_start_header);
    if calculated_crc != start_header_crc {
        return Err(nom::Err::Failure(SevenZParserError::Crc(
            start_header_crc,
            calculated_crc,
        )));
    }

    return Ok((
        input,
        SignatureHeader {
            archive_version,
            start_header_crc,
            start_header,
        },
    ));
}

pub fn property_id(input: &[u8]) -> IResult<&[u8], PropertyID, SevenZParserError<&[u8]>> {
    let (input, p_u8) = u8(input)?;
    match PropertyID::try_from(p_u8) {
        Ok(p) => return Ok((input, p)),
        Err(_) => {
            return Err(nom::Err::Failure(SevenZParserError::InvalidPropertyID(
                p_u8,
            )))
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn archive_property(
    input: &[u8],
) -> IResult<&[u8], (PropertyID, &[u8]), SevenZParserError<&[u8]>> {
    let (input, prop_type) = property_id(input)?;
    let (input, len) = to_err!(sevenz_uint64(input));
    let len_usize: usize = to_usize_or_err!(len);
    let (input, prop_data) = take(len_usize)(input)?;
    return Ok((input, (prop_type, prop_data)));
}
pub fn archive_properties(
    input: &[u8],
) -> IResult<&[u8], ArchiveProperties, SevenZParserError<&[u8]>> {
    let (input, _) = tag([PropertyID::ArchiveProperties as u8])(input)?;
    let mut props: Vec<(PropertyID, Vec<u8>)> = vec![];
    loop {
        let (input, (prop_id, prop_data)) = archive_property(input)?;
        if prop_id == PropertyID::End {
            return Ok((
                input,
                ArchiveProperties {
                    property_data: props,
                },
            ));
        }

        props.push((prop_id, Vec::from(prop_data)));
    }
}

pub fn header(input: &[u8]) -> IResult<&[u8], (), SevenZParserError<&[u8]>> {
    let (input, archive_properties) = archive_properties(input)?;
    return Ok((input, ()));
}

/*
pub fn archive(input: &[u8]) -> IResult<&[u8], Archive> {
    let (input, signature_header) = to_err!(signature_header(input));
    return Ok((
        input,
        Archive {
            signature_header,
            packed_streams,
            packed_streams_for_headers,
            header_or_packed_header,
        },
    ));
}
*/
