use super::*;

use alloc::vec;
use alloc::vec::Vec;

/// The goal of this type is to get coder and stream information into a form where
/// it can be easily iterated over to determine how to get at each file.
#[derive(Debug, Clone, PartialEq)]
pub struct FileStreamInfo {
    pub coder: Coder,
    pub offset: u64,
    pub size: u64,
    pub crc: u32,
}

/// Retrieve all `FileStreamInfo` (one for each file in the archive).
///
/// TODO: Support complex coders
pub fn get_file_streams_info(ci: &CodersInfo) -> Vec<FileStreamInfo> {
    let folders = match ci.folders_or_data_stream_index.clone() {
        Right(folders) => folders,
        Left(_) => panic!("CodersInfo with external folders is currently unsupported!"),
    };

    let mut res = vec![];
    // Assumption: Files belong to folders in the order they appear (spec doesn't say anything about this)
    // TODO: Deal with substreams
    let mut i = 0;
    for folder in folders {
        for coder in folder.coders {
            for _ in 0..coder.get_num_out_streams() {
                // FIXME: Retrieve from substreams if needed
                // let crc = ci.folders_unpack_digests.clone().unwrap()[i];
                let crc = 0;
                res.push(FileStreamInfo {
                    coder: coder.clone(),
                    // FIXME: Actually calculate
                    offset: 0,
                    size: ci.streams_unpack_sizes[i],
                    crc,
                });
            }
        }
        i += 1;
    }
    return res;
}

/// Turn the vector into one that repeats it's
#[cfg(test)]
mod test {
    use super::types;
    use alloc::vec;

    #[test]
    pub fn get_file_streams_info() {
        let ci = types::CodersInfo {
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
        let expected = vec![super::FileStreamInfo {
            coder: types::Coder {
                complex: None,
                attrs: None,
                id: vec![33, 1],
            },
            offset: 0,
            size: 15,
            crc: 0,
        }];

        let res = super::get_file_streams_info(&ci);

        assert_eq!(res, expected);
    }
}
