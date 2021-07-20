//! Custom nom parsers for the 7z format

extern crate std;

use super::crc;
use super::err::*;
use super::types::*;

use alloc::vec;
use alloc::vec::*;
use core::convert::*;

use either::*;
use nom::bytes::complete::{tag, take};
use nom::combinator::cond;
use nom::error::context;
use nom::multi::{count, length_count};
use nom::number::complete::{le_u32, le_u64, u8};
use nom::IResult;

/// Header magic bytes
const MAGIC: [u8; 6] = [b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

/// 7zip uses a weird packed integer format to represent some u64 values.
/// Parse that and convert it to a normal u64 in native endianness.
pub fn sevenz_uint64(input: &[u8]) -> IResult<&[u8], u64, SevenZParserError<&[u8]>> {
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
pub fn sevenz_uint64_as_usize(input: &[u8]) -> IResult<&[u8], usize, SevenZParserError<&[u8]>> {
    let (input, as_u64) = context("sevenz_uint64_as_usize as_u64", sevenz_uint64)(input)?;
    let as_usize = crate::to_usize_or_err!(as_u64);
    return Ok((input, as_usize));
}

pub fn archive_version(input: &[u8]) -> IResult<&[u8], ArchiveVersion, SevenZParserError<&[u8]>> {
    let (input, major) = context("archive_version major", u8)(input)?;
    let (input, minor) = context("archive_version minor", u8)(input)?;
    return Ok((input, ArchiveVersion { major, minor }));
}

pub fn start_header(input: &[u8]) -> IResult<&[u8], StartHeader, SevenZParserError<&[u8]>> {
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

pub fn signature_header(input: &[u8]) -> IResult<&[u8], SignatureHeader, SevenZParserError<&[u8]>> {
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

pub fn tag_property_id(
    input: &[u8],
    id: PropertyID,
) -> IResult<&[u8], PropertyID, SevenZParserError<&[u8]>> {
    let (input, p) = context("tag_property_id", property_id)(input)?;
    if p == id {
        return Ok((input, p));
    } else {
        return Err(nom::Err::Failure(SevenZParserError::new(
            SevenZParserErrorKind::Nom(input, nom::error::ErrorKind::Tag),
        )));
    }
}

#[allow(clippy::type_complexity)]
pub fn archive_property(
    input: &[u8],
) -> IResult<&[u8], (PropertyID, &[u8]), SevenZParserError<&[u8]>> {
    let (input, prop_type) = context("archive_property prop_type", property_id)(input)?;
    let (input, len) = context("archive_property len", sevenz_uint64_as_usize)(input)?;
    let (input, prop_data) = context("archive_property prop_data", take(len))(input)?;
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

/// Runs the second parser and returns it's output/error if the first parser succeeds.
/// Doesn't run the second parser and returns Ok((input, None)) if the first parser fails.
pub fn preceded_opt<I, O1, O2, E: nom::error::ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, Option<O2>, E>
where
    F: nom::Parser<I, O1, E>,
    G: nom::Parser<I, O2, E>,
    I: Clone,
{
    move |input: I| match first.parse(input.clone()) {
        Ok((input, _)) => {
            let (input, val) = second.parse(input)?;
            return Ok((input, Some(val)));
        }
        Err(_) => Ok((input, None)),
    }
}

pub fn pack_info(input: &[u8]) -> IResult<&[u8], PackInfo, SevenZParserError<&[u8]>> {
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

pub fn bool_byte(input: &[u8]) -> IResult<&[u8], bool, SevenZParserError<&[u8]>> {
    let (input, byte) = context("bool_byte byte", u8)(input)?;
    return match byte {
        0 => Ok((input, false)),
        1 => Ok((input, true)),
        _ => Err(nom::Err::Failure(SevenZParserError::new(
            SevenZParserErrorKind::InvalidBooleanByte(byte),
        ))),
    };
}

pub fn coder(input: &[u8]) -> IResult<&[u8], Coder, SevenZParserError<&[u8]>> {
    fn is_complex(props: u8) -> bool {
        (props & 0b0000_1000) > 0
    }
    fn has_attrs(props: u8) -> bool {
        (props & 0b0000_0100) > 0
    }
    fn id_len(props: u8) -> usize {
        ((props & 0b1111_0000) >> 4) as usize
    }

    // TODO: Error for illegally set bit 7

    let (input, props) = context("coder properties", u8)(input)?;
    let (input, id) = context("coder ID", take(id_len(props)))(input)?;
    let id = Vec::from(id);

    let mut input_mut = input;
    let mut complex = None;
    if is_complex(props) {
        let (input, num_in_streams) = context("coder num_in_streams", sevenz_uint64)(input)?;
        let (input, num_out_streams) = context("coder num_out_streams", sevenz_uint64)(input)?;
        complex = Some(CoderComplex {
            num_in_streams,
            num_out_streams,
        });
        input_mut = input;
    }

    let mut attrs = None;
    if has_attrs(props) {
        let (input, attr_size) = context("coder attr_size", sevenz_uint64_as_usize)(input_mut)?;
        let (input, attrs_slice) = context("coder attrs", take(attr_size))(input)?;

        attrs = Some(Vec::from(attrs_slice));
        input_mut = input;
    }
    let input = input_mut;

    return Ok((input, Coder { complex, attrs, id }));
}

pub fn folder_coders(input: &[u8]) -> IResult<&[u8], Vec<Coder>, SevenZParserError<&[u8]>> {
    let (input, coders_vec) = context(
        "folder_coders coders",
        length_count(
            context("folder_coders num_coders", sevenz_uint64_as_usize),
            context("folder_coders coders", coder),
        ),
    )(input)?;
    return Ok((input, coders_vec));
}

pub fn folder(input: &[u8]) -> IResult<&[u8], Folder, SevenZParserError<&[u8]>> {
    let (input, coders_vec) = context("folder coders", folder_coders)(input)?;

    let num_out_streams_total: u64 = coders_vec
        .iter()
        .map(|x| {
            if x.complex.is_none() {
                1
            } else {
                x.complex.unwrap().num_out_streams
            }
        })
        .sum();
    let num_out_streams_total: usize = crate::to_usize_or_err!(num_out_streams_total);

    let num_bind_pairs = num_out_streams_total - 1;
    let mut input_mut = input;
    let mut bind_pairs: Vec<(u64, u64)> = vec![];
    bind_pairs.reserve(num_bind_pairs);
    for _ in 0..num_bind_pairs {
        let (input, in_index) = context("folder bind pair in_index", sevenz_uint64)(input)?;
        let (input, out_index) = context("folder bind pair out_index", sevenz_uint64)(input)?;
        input_mut = input;
        bind_pairs.push((in_index, out_index));
    }

    let num_in_streams_total: u64 = coders_vec
        .iter()
        .map(|x| {
            if x.complex.is_none() {
                1
            } else {
                x.complex.unwrap().num_in_streams
            }
        })
        .sum();
    let num_in_streams_total: usize = crate::to_usize_or_err!(num_in_streams_total);
    let num_packed_streams = num_in_streams_total - num_bind_pairs;

    let (input, packed_streams_indices) = context(
        "folder packed_streams_indices",
        cond(
            num_packed_streams > 1,
            count(sevenz_uint64, num_packed_streams),
        ),
    )(input)?;

    return Ok((
        input,
        Folder {
            coders: coders_vec,
            bind_pairs,
            packed_streams_indices,
        },
    ));
}

pub fn take_folders(
    input: &[u8],
    num_folders: usize,
) -> IResult<&[u8], Vec<Folder>, SevenZParserError<&[u8]>> {
    let mut input_mut = input;
    let mut folders = Vec::new();
    folders.reserve(num_folders);
    for _ in 0..num_folders {
        let (input, one_folder) = context("folders folder", folder)(input_mut)?;
        input_mut = input;
        folders.push(one_folder);
    }
    let input = input_mut;

    return Ok((input, folders));
}

pub fn coders_info(input: &[u8]) -> IResult<&[u8], CodersInfo, SevenZParserError<&[u8]>> {
    let (input, _) = context(
        "coders_info PropertyID::UnPackInfo",
        tag([PropertyID::UnPackInfo as u8]),
    )(input)?;
    let (input, _) = context(
        "coders_info PropertyID::Folder",
        tag([PropertyID::Folder as u8]),
    )(input)?;

    let (input, num_folders) = context("coders_info num_folders", sevenz_uint64_as_usize)(input)?;

    let (input, external) = context("coders_info external", bool_byte)(input)?;
    let folders_or_data_stream_index;
    let mut input_mut = input;
    if external {
        let (input, data_stream_index) =
            context("coders_info data_stream_index", sevenz_uint64)(input_mut)?;
        folders_or_data_stream_index = Right(data_stream_index);
        input_mut = input;
    } else {
        let (input, folders) =
            context("coders_info folders", count(folder, num_folders))(input_mut)?;
        folders_or_data_stream_index = Left(folders);
        input_mut = input;
    }
    let input = input_mut;

    let (input, _) = context(
        "coders_info PropertyID::CodersUnPackSize",
        tag([PropertyID::CodersUnPackSize as u8]),
    )(input)?;

    return Ok((
        input,
        CodersInfo {
            num_folders,
            folders_or_data_stream_index,
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
