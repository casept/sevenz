use super::combinators;
use super::types;

#[test]
fn archive_version() {
    let data = include_bytes!("../../testdata/test.txt.7z");
    let (_, res) = combinators::archive_version(&data[6..]).unwrap();

    let expected = types::ArchiveVersion { major: 0, minor: 4 };

    assert_eq!(res, expected);
}

#[test]
fn signature_header() {
    let data = include_bytes!("../../testdata/test.txt.7z");
    let (_, res) = combinators::signature_header(data).unwrap();

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
        let (remainder, res) = combinators::sevenz_uint64(input).unwrap();

        assert_eq!(res, *expected);
        assert_eq!(remainder.len(), *expected_len_remaining);
    }
}

#[test]
fn header() {
    let input = include_bytes!("../../testdata/test.txt.7z");
    // Already tested elsewhere, just here to skip ahead enough bytes
    let (input, _) = combinators::signature_header(input).unwrap();
    // From here, header should be in 19 bytes
    let input = &input[19..];
    let (input, hdr) = combinators::header(input).unwrap();
}
