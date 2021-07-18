use sevenz;

#[test]
fn test_unpack_single_file() {
    let archive = include_bytes!("../testdata/test.7z");
    let expected = include_bytes!("../testdata/test.txt");

    let res = sevenz::read::from_bytes(archive);
    assert_eq!(res, expected);
}
