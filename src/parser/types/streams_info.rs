use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SubStreamsInfo {
    pub num_unpack_streams_in_folders: Option<Vec<u64>>,
    pub unpack_sizes: Option<Vec<u64>>,
    pub unknown_digests: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamsInfo {
    pub pack_info: Option<PackInfo>,
    pub coders_info: Option<CodersInfo>,
    pub substreams_info: Option<SubStreamsInfo>,
}
