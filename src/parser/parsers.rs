//! Custom nom parsers for the 7z format

extern crate std;

use super::crc;
use super::err::*;
use super::types::*;

use alloc::vec;
use alloc::vec::*;
use core::convert::*;

use nom::bytes::complete::{tag, take};
use nom::error::context;
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

pub fn archive_version(input: &[u8]) -> IResult<&[u8], ArchiveVersion> {
    let (input, major) = context("archive_version major", u8)(input)?;
    let (input, minor) = context("archive_version minor", u8)(input)?;
    return Ok((input, ArchiveVersion { major, minor }));
}

pub fn start_header(input: &[u8]) -> IResult<&[u8], StartHeader> {
    let (input, next_header_offset) =
        context("start_header next_header_offset", u64(Endianness::Little))(input)?;
    let (input, next_header_size) =
        context("start_header next_header_size", u64(Endianness::Little))(input)?;
    let (input, next_header_crc) =
        context("start_header next_header_crc", u32(Endianness::Little))(input)?;
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
    let (input, _) = context("signature_header magic bytes", tag(MAGIC))(input)?;
    // This is super ugly, but not sure how to solve this more elegantly
    let (input, archive_version) =
        crate::to_err!(context("signature_header archive_version", archive_version)(input));
    let (input, start_header_crc) =
        context("signature_header start_header_crc", u32(Endianness::Little))(input)?;
    let (_, raw_start_header) = context(
        "signature_header raw_start_header",
        take(START_HEADER_SIZE_BYTES),
    )(input)?;
    let (input, start_header) = crate::to_err!(context(
        "signature_header start_header",
        start_header
    )(input));

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

pub fn property_id(input: &[u8]) -> IResult<&[u8], PropertyID, SevenZParserError<&[u8]>> {
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

#[allow(clippy::type_complexity)]
pub fn archive_property(
    input: &[u8],
) -> IResult<&[u8], (PropertyID, &[u8]), SevenZParserError<&[u8]>> {
    let (input, prop_type) = context("archive_property prop_type", property_id)(input)?;
    let (input, len) = crate::to_err!(context("archive_property len", sevenz_uint64)(input));
    let len_usize: usize = crate::to_usize_or_err!(len);
    let (input, prop_data) = context("archive_property prop_data", take(len_usize))(input)?;
    return Ok((input, (prop_type, prop_data)));
}
pub fn archive_properties(
    input: &[u8],
) -> IResult<&[u8], ArchiveProperties, SevenZParserError<&[u8]>> {
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

pub fn pack_sizes(
    input: &[u8],
    num_pack_sizes: usize,
) -> IResult<&[u8], Vec<u64>, SevenZParserError<&[u8]>> {
    let (input, _) = context("pack_info PropertyID::Size", tag([PropertyID::Size as u8]))(input)?;
    let mut sizes: Vec<u64> = vec![];
    sizes.reserve(num_pack_sizes);
    let mut input_mut: &[u8] = input;
    for _ in 0..num_pack_sizes {
        let (input, pack_size) =
            crate::to_err!(context("pack_sizes pack_size", sevenz_uint64)(input_mut));
        sizes.push(pack_size);
        input_mut = input;
    }
    return Ok((input_mut, sizes));
}

pub fn pack_crcs(
    input: &[u8],
    num_crcs: usize,
) -> IResult<&[u8], Vec<u32>, SevenZParserError<&[u8]>> {
    let (input, _) = context("pack_crcs PropertyID::CRC", tag([PropertyID::CRC as u8]))(input)?;
    let mut crcs: Vec<u32> = vec![];
    crcs.reserve(num_crcs);
    let mut input_mut: &[u8] = input;
    for _ in 0..num_crcs {
        let (input, crc) = crate::to_err!(context(
            "pack_crcs pack_stream_digests",
            u32(Endianness::Little)
        )(input_mut));
        crcs.push(crc);
        input_mut = input;
    }
    return Ok((input_mut, crcs));
}

pub fn pack_info(input: &[u8]) -> IResult<&[u8], PackInfo, SevenZParserError<&[u8]>> {
    let (input, _) = context(
        "pack_info PropertyID::PackInfo",
        tag([PropertyID::PackInfo as u8]),
    )(input)?;
    let (input, pack_pos) = crate::to_err!(context("pack_info pack_pos", sevenz_uint64)(input));
    let (input, num_pack_streams) =
        crate::to_err!(context("pack_info num_pack_streams", sevenz_uint64)(input));
    let num_pack_streams_usize = crate::to_usize_or_err!(num_pack_streams);
    // TODO: Confirm that this is the actual criteria (docs are very vague)
    if num_pack_streams == 0 {
        return Ok((
            input,
            PackInfo {
                pack_pos,
                num_pack_streams,
                sizes: None,
                crcs: None,
            },
        ));
    }

    let (input, sizes) =
        context("pack_info sizes", |x| pack_sizes(x, num_pack_streams_usize))(input)?;
    let (input, crcs) = context("pack_info crcs", |x| pack_crcs(x, num_pack_streams_usize))(input)?;

    return Ok((
        input,
        PackInfo {
            pack_pos,
            num_pack_streams,
            sizes: Some(sizes),
            crcs: Some(crcs),
        },
    ));
}

pub fn streams_info(input: &[u8]) -> IResult<&[u8], (), SevenZParserError<&[u8]>> {
    let (input, _) = context("streams_info pack_info", pack_info)(input)?;
    // TODO:
    return Ok((input, ()));
}

pub fn header(input: &[u8]) -> IResult<&[u8], (), SevenZParserError<&[u8]>> {
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
            std::println!("Segment prop ID: {:?}", segment);
            panic!("This has to be fixed");
        }
    }
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
