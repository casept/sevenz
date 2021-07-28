extern crate std;
use super::super::parsers;
use super::super::types;

use alloc::vec;
use bitvec::prelude::*;
use either::*;
use std::prelude::rust_2018::*;

const UNCOMPRESSED_ARCHIVE: &[u8] = include_bytes!("../../../testdata/test-uncompressed.txt.7z");

#[test]
fn archive_version() {
    let input = UNCOMPRESSED_ARCHIVE;
    let (_, res) = parsers::archive_version(&input[6..]).unwrap();

    let expected = types::ArchiveVersion { major: 0, minor: 4 };

    assert_eq!(res, expected);
}

#[test]
fn signature_header() {
    let input = UNCOMPRESSED_ARCHIVE;
    let (_, res) = parsers::signature_header(input).unwrap();

    let expected_version = types::ArchiveVersion { major: 0, minor: 4 };
    let expected_start_header = types::StartHeader {
        next_header_offset: 19,
        next_header_size: 90,
        next_header_crc: 970299701,
    };
    let expected_res = types::SignatureHeader {
        archive_version: expected_version,
        start_header_crc: 9174449,
        start_header: expected_start_header,
    };

    assert_eq!(res, expected_res);
}

#[test]
fn sevenz_uint64() {
    let test_cases: &[([u8; 8], u64, usize)] = &[
        ([0; 8], 0, 7),
        ([0b0111_1111; 8], 127, 7),
        ([0b1000_0000, 42, 0, 0, 0, 0, 0, 0], 42, 6),
        ([0b1000_1111, 42, 0, 0, 0, 0, 0, 0], 3882, 6),
    ];

    for (input, expected, expected_len_remaining) in test_cases {
        let (remainder, res) = parsers::sevenz_uint64(input).unwrap();

        assert_eq!(res, *expected);
        assert_eq!(remainder.len(), *expected_len_remaining);
    }
}

#[test]
fn take_bitvec() {
    let test_cases: &[([u8; 10], usize, usize, BitVec)] = &[
        ([0; 10], 0, 10, bitvec![]),
        (
            [0b0001_0000, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            4,
            9,
            bitvec![0, 0, 0, 1],
        ),
        (
            [0b0000_0001, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            8,
            9,
            bitvec![0, 0, 0, 0, 0, 0, 0, 1],
        ),
        (
            [0b1011_0000, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            4,
            9,
            bitvec![1, 0, 1, 1],
        ),
        (
            [0b1000_0010, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            8,
            9,
            bitvec![1, 0, 0, 0, 0, 0, 1, 0],
        ),
        (
            [0b0101_1111, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            5,
            9,
            bitvec![0, 1, 0, 1, 1],
        ),
        (
            [0b1000_0010, 0b0010_0010, 0, 0, 0, 0, 0, 0, 0, 0],
            16,
            8,
            bitvec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0],
        ),
        (
            [0b1000_0010, 0b0010_0010, 0b1001_1111, 0, 0, 0, 0, 0, 0, 0],
            20,
            7,
            bitvec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1],
        ),
        (
            [
                0b1000_0010,
                0b0010_0010,
                0b1001_1111,
                0,
                0,
                0,
                0,
                0b0101_1011,
                0b1011_0100,
                0,
            ],
            72,
            1,
            bitvec![
                1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0
            ],
        ),
    ];

    for (input, num_bits, bytes_remaining, expected) in test_cases {
        let (remainder, res) = parsers::take_bitvec(input, *num_bits).unwrap();
        assert_eq!(remainder.len(), *bytes_remaining);
        assert_eq!(res, *expected);
    }
}
#[test]
fn pack_info() {
    let input = UNCOMPRESSED_ARCHIVE;
    let expected = types::PackInfo {
        pack_pos: 0,
        num_pack_streams: 1,
        sizes: Some(vec![19]),
        crcs: None,
    };

    // Cut parts not relevant here
    let input = &input[53..];
    let (_, res) = parsers::pack_info(input).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn coders_info() {
    let input = UNCOMPRESSED_ARCHIVE;
    let expected = types::CodersInfo {
        num_folders: 1,
        folders_or_data_stream_index: either::Right(vec![types::Folder {
            coders: vec![types::Coder {
                complex: None,
                attrs: None,
                id: vec![33, 1],
            }],
            bind_pairs: vec![],
            packed_streams_indices: Some(vec![0]),
        }]),
        streams_unpack_sizes: vec![15],
        folders_unpack_digests: None,
    };

    // Cut parts not relevant here
    let input = &input[59..];
    let (_, res) = parsers::coders_info(input).unwrap();

    assert_eq!(res, expected);
}

#[test]
fn substreams_info() {
    let input = UNCOMPRESSED_ARCHIVE;
    let expected = types::SubStreamsInfo {
        num_unpack_streams_in_folders: None,
        unpack_sizes: None,
        unknown_digests: Some(vec![3224210433, 83886330, 792833]),
    };

    // Cut parts not relevant here
    let input = &input[71..];
    let (_, res) = parsers::substreams_info(input, 1, 3).unwrap();

    assert_eq!(res, expected);
}

#[test]
fn streams_info() {
    let input = UNCOMPRESSED_ARCHIVE;
    // Cut parts not relevant here
    let input = &input[53..];

    let (_, res) = parsers::streams_info(input, None).unwrap();
}

#[test]
fn files_info() {
    let input = UNCOMPRESSED_ARCHIVE;
    let expected = types::FilesInfo {
        num_files: 1,
        properties: vec![
            types::FilesProperty::Names(vec![Right(String::from("test.txt"))]),
            types::FilesProperty::ATime(vec![Some(Right(132708606310000000))]),
            types::FilesProperty::Attributes(vec![Some(Right(2175041568))]),
        ],
    };
    // Cut parts not relevant here
    let input = &input[80..];

    let (_, res) = parsers::files_info(input, 0).unwrap();
    assert_eq!(res, expected);
}

#[test]
fn header() {
    let input = UNCOMPRESSED_ARCHIVE;
    // Already tested elsewhere, just here to skip ahead enough bytes
    let (input, _) = parsers::signature_header(input).unwrap();
    // From here, header should be in 19 bytes
    let input = &input[19..];
    let (input, hdr) = parsers::header(input).unwrap();
}

#[test]
fn archive() {
    let input = UNCOMPRESSED_ARCHIVE;
    let expected = types::Archive {
        signature_header: types::SignatureHeader {
            archive_version: types::ArchiveVersion { major: 0, minor: 4 },
            start_header_crc: 9174449,
            start_header: types::StartHeader {
                next_header_offset: 19,
                next_header_size: 90,
                next_header_crc: 970299701,
            },
        },
        header_or_packed_header: Left(types::Header {
            archive_properties: None,
            additional_streams: None,
            main_streams: Some(types::StreamsInfo {
                pack_info: Some(types::PackInfo {
                    pack_pos: 0,
                    num_pack_streams: 1,
                    sizes: Some(vec![19]),
                    crcs: None,
                }),
                coders_info: Some(types::CodersInfo {
                    num_folders: 1,
                    folders_or_data_stream_index: Right(vec![types::Folder {
                        coders: vec![types::Coder {
                            complex: None,
                            attrs: None,
                            id: vec![33, 1],
                        }],
                        bind_pairs: vec![],
                        packed_streams_indices: Some(vec![0]),
                    }]),
                    streams_unpack_sizes: vec![15],
                    folders_unpack_digests: None,
                }),
                substreams_info: Some(types::SubStreamsInfo {
                    num_unpack_streams_in_folders: None,
                    unpack_sizes: None,
                    unknown_digests: Some(vec![3224210433, 83886330, 792833]),
                }),
            }),
            files: Some(types::FilesInfo {
                num_files: 1,
                properties: vec![
                    types::FilesProperty::Names(vec![Right(String::from("test.txt"))]),
                    types::FilesProperty::ATime(vec![Some(Right(132708606310000000))]),
                    types::FilesProperty::Attributes(vec![Some(Right(2175041568))]),
                ],
            }),
        }),
    };

    let (_, res) = parsers::archive(input).unwrap();

    assert_eq!(res, expected);
}
