//! Custom nom parsers for the 7z format

mod bit;
pub use bit::*;
mod sevenz_uint64;
pub use sevenz_uint64::*;
mod files_info;
pub use files_info::*;
mod coders_info;
pub use coders_info::*;
mod streams_info;
pub use streams_info::*;
mod header;
pub use header::*;
#[cfg(test)]
mod test;

use super::combinators::*;
use super::crc;
use super::err::*;
use super::types::*;

use alloc::vec;
use alloc::vec::*;
use core::convert::*;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::combinator::{cond, map, opt};
use nom::error::context;
use nom::multi::{count, length_count, many_till};
use nom::number::complete::{le_u16, le_u32, le_u64, u8};
use nom::sequence::pair;

/// Header magic bytes
const MAGIC: [u8; 6] = [b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

/// Error type that all parsers return.
pub type SevenZResult<'a, T> = nom::IResult<&'a [u8], T, SevenZParserError<&'a [u8]>>;

pub fn archive_version(input: &[u8]) -> SevenZResult<ArchiveVersion> {
    let (input, major) = context("archive_version major", u8)(input)?;
    let (input, minor) = context("archive_version minor", u8)(input)?;
    return Ok((input, ArchiveVersion { major, minor }));
}

pub fn start_header(input: &[u8]) -> SevenZResult<StartHeader> {
    let (input, next_header_offset) = context("start_header next_header_offset", le_u64)(input)?;
    let (input, next_header_size) = context("start_header next_header_size", le_u64)(input)?;
    let (input, next_header_crc) = context("start_header next_header_crc", le_u32)(input)?;
    return Ok((
        input,
        StartHeader {
            next_header_offset,
            next_header_size,
            next_header_crc,
        },
    ));
}

pub fn signature_header(input: &[u8]) -> SevenZResult<SignatureHeader> {
    let (input, _) = context("signature_header magic bytes", tag(MAGIC))(input)?;
    let (input, archive_version) =
        context("signature_header archive_version", archive_version)(input)?;
    let (input, start_header_crc) = context("signature_header start_header_crc", le_u32)(input)?;
    let (_, raw_start_header) = context(
        "signature_header raw_start_header",
        take(START_HEADER_SIZE_BYTES),
    )(input)?;
    let (input, start_header) = context("signature_header start_header", start_header)(input)?;

    // Verify CRC
    let calculated_crc = crc::sevenz_crc(raw_start_header);
    if calculated_crc != start_header_crc {
        return Err(nom::Err::Failure(SevenZParserError::new(
            SevenZParserErrorKind::Crc(start_header_crc, calculated_crc),
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

pub fn bool_byte(input: &[u8]) -> SevenZResult<bool> {
    let (input, byte) = context("bool_byte byte", u8)(input)?;
    return match byte {
        0 => Ok((input, false)),
        1 => Ok((input, true)),
        _ => Err(nom::Err::Error(SevenZParserError::new(
            SevenZParserErrorKind::InvalidBooleanByte(byte),
        ))),
    };
}

pub fn property_id(input: &[u8]) -> SevenZResult<PropertyID> {
    let (input, p_u8) = context("property_id", u8)(input)?;
    match PropertyID::try_from(p_u8) {
        Ok(p) => return Ok((input, p)),
        Err(_) => {
            return Err(nom::Err::Error(SevenZParserError::new(
                SevenZParserErrorKind::InvalidPropertyID(p_u8),
            )))
        }
    }
}

pub fn tag_property_id(input: &[u8], id: PropertyID) -> SevenZResult<PropertyID> {
    let (input, p) = context("tag_property_id", property_id)(input)?;
    if p == id {
        return Ok((input, p));
    } else {
        return Err(nom::Err::Error(SevenZParserError::new(
            SevenZParserErrorKind::Nom(input, nom::error::ErrorKind::Tag),
        )));
    }
}
