use sevenz;

const UNCOMPRESSED_TEST_ARCHIVE: &[u8] = include_bytes!("../testdata/test-uncompressed.txt.7z");
const TEST_TXT_CONTENTS: &[u8] = include_bytes!("../testdata/test-uncompressed.txt");

#[test]
fn unpack_single_uncompressed_file() {
    let res = sevenz::read::extract_file("test.txt", UNCOMPRESSED_TEST_ARCHIVE).unwrap();
    assert_eq!(res, TEST_TXT_CONTENTS);
}
