use alloc::vec::Vec;

/// An abstraction over the low-level 7zip format archive construct.
///
/// This is the "top-level" type that the parser emits in the end.
#[derive(Debug, Clone)]
pub struct HighLevelArchive {
    pub files: Vec<super::File>,
}

impl HighLevelArchive {
    pub fn from_low_level_archive(ar: &super::types::Archive) -> HighLevelArchive {
        let hdr = ar
            .header_or_packed_header
            .clone()
            .expect_left("Archives with packed headers not yet supported");
        let mut files: Vec<super::File> = Vec::new();
        for i in 0..hdr.files.clone().unwrap().num_files {
            files.push(super::File::from_files_and_coders_info(
                &hdr.files.clone().unwrap(),
                &hdr.main_streams.clone().unwrap().coders_info.unwrap(),
                i,
            ));
        }

        return HighLevelArchive { files };
    }
}
