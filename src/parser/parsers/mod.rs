//! Custom nom parsers for the 7z format

mod bit;
pub use bit::*;
mod sevenz_uint64;
pub use sevenz_uint64::*;
mod files_info;
pub use files_info::*;
mod coders_info;
pub use coders_info::*;
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
        _ => Err(nom::Err::Failure(SevenZParserError::new(
            SevenZParserErrorKind::InvalidBooleanByte(byte),
        ))),
    };
}

pub fn property_id(input: &[u8]) -> SevenZResult<PropertyID> {
    let (input, p_u8) = context("property_id", u8)(input)?;
    match PropertyID::try_from(p_u8) {
        Ok(p) => return Ok((input, p)),
        Err(_) => {
            return Err(nom::Err::Failure(SevenZParserError::new(
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
        return Err(nom::Err::Failure(SevenZParserError::new(
            SevenZParserErrorKind::Nom(input, nom::error::ErrorKind::Tag),
        )));
    }
}

pub fn archive_property(input: &[u8]) -> SevenZResult<(PropertyID, &[u8])> {
    let (input, prop_type) = context("archive_property prop_type", property_id)(input)?;
    let (input, len) = context("archive_property len", sevenz_uint64_as_usize)(input)?;
    let (input, prop_data) = context("archive_property prop_data", take(len))(input)?;
    return Ok((input, (prop_type, prop_data)));
}

pub fn archive_properties(input: &[u8]) -> SevenZResult<ArchiveProperties> {
    let (input, _) = context(
        "archive_properties PropertyID::ArchiveProperties",
        tag([PropertyID::ArchiveProperties as u8]),
    )(input)?;
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

pub fn pack_info(input: &[u8]) -> SevenZResult<PackInfo> {
    let (input, _) = context(
        "pack_info PropertyID::PackInfo",
        tag([PropertyID::PackInfo as u8]),
    )(input)?;
    let (input, pack_pos) = context("pack_info pack_pos", sevenz_uint64)(input)?;
    let (input, num_pack_streams) =
        context("pack_info num_pack_streams", sevenz_uint64_as_usize)(input)?;

    // TODO: The spec is not exactly clear about the circumstances under which these 2 are optional.
    // TODO: For now, let's just assume that they're optional when their markers are present and vice versa.
    let (input, sizes) = context(
        "pack_info sizes",
        preceded_opt(
            |x| tag_property_id(x, PropertyID::Size),
            count(sevenz_uint64, num_pack_streams),
        ),
    )(input)?;
    let (input, crcs) = context(
        "pack_info crcs",
        preceded_opt(
            |x| tag_property_id(x, PropertyID::CRC),
            count(le_u32, num_pack_streams),
        ),
    )(input)?;

    let (input, _) = context("pack_info PropertyID::End", tag([PropertyID::End as u8]))(input)?;
    return Ok((
        input,
        PackInfo {
            pack_pos,
            num_pack_streams,
            sizes: sizes,
            crcs: crcs,
        },
    ));
}

pub fn substreams_info(input: &[u8], num_folders: usize) -> SevenZResult<SubStreamsInfo> {
    let (input, _) = context(
        "substreams_info PropertyID::SubStreamsInfo",
        tag([PropertyID::SubStreamsInfo as u8]),
    )(input)?;

    let (input, num_unpack_streams_in_folders) = context(
        "coders_info num_unpack_streams_in_folders",
        preceded_opt(
            tag([PropertyID::NumUnPackStream as u8]),
            count(sevenz_uint64, num_folders),
        ),
    )(input)?;

    let (input, unpack_sizes) = context(
        "coders_info unpack_sizes",
        preceded_opt(
            tag([PropertyID::Size as u8]),
            count(sevenz_uint64, {
                // FIXME: Don't panic
                //let total_streams: u64 = num_unpack_streams_in_folders.unwrap().iter().sum();
                let total_streams: u64 = 1;
                to_usize_or_err!(total_streams)
            }),
        ),
    )(input)?;

    // FIXME: Have to figure out how to calculate number of streams with unknown CRC
    let (input, unknown_crcs) = context(
        "coders_info unknown_crcs",
        preceded_opt(
            tag([PropertyID::CRC as u8]),
            count(sevenz_uint64, to_usize_or_err!(1)),
        ),
    )(input)?;

    let (input, _) = context(
        "substreams_info PropertyID::End",
        tag([PropertyID::End as u8]),
    )(input)?;

    return Ok((input, SubStreamsInfo {}));
}

pub fn streams_info(input: &[u8]) -> SevenZResult<StreamsInfo> {
    // FIXME: Pass proper num_folders
    let (input, pack_info_data) = context("streams_info pack_info", opt(pack_info))(input)?;
    let (input, coders_info_data) = context("streams_info coders_info", opt(coders_info))(input)?;
    let (input, substreams_info_data) = context(
        "streams_info substreams_info",
        opt(|x| substreams_info(x, 1)),
    )(input)?;
    let (input, _) = context("streams_info PropertyID::End", tag([PropertyID::End as u8]))(input)?;

    // TODO:
    return Ok((
        input,
        StreamsInfo {
            pack_info: pack_info_data,
            coders_info: coders_info_data,
            substreams_info: substreams_info_data,
        },
    ));
}

pub fn header(input: &[u8]) -> SevenZResult<()> {
    let (input, _) = context("header PropertyID::Header", tag([PropertyID::Header as u8]))(input)?;
    let (input, segment) = property_id(input)?;
    use PropertyID::*;
    match segment {
        ArchiveProperties => {
            let (input, archive_properties) =
                context("header archive_properties", archive_properties)(input)?;
        }
        MainStreamsInfo => {
            let (input, main_streams_info) =
                context("header main_streams_info", streams_info)(input)?;
        }
        _ => {
            panic!("This has to be fixed");
        }
    }
    return Ok((input, ()));
}

/*
pub fn archive(input: &[u8]) -> SevenZResult<Archive> {
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
