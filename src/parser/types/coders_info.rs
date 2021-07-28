use core::convert::TryInto;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoderComplex {
    pub num_in_streams: u64,
    pub num_out_streams: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coder {
    pub complex: Option<CoderComplex>,
    pub attrs: Option<Vec<u8>>,
    pub id: Vec<u8>,
}

impl Coder {
    pub fn get_num_out_streams(&self) -> usize {
        return match self.complex {
            Some(n) => n.num_out_streams.try_into().unwrap(),
            None => 1,
        };
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    pub coders: Vec<Coder>,
    pub bind_pairs: Vec<(u64, u64)>,
    pub packed_streams_indices: Option<Vec<u64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodersInfo {
    pub num_folders: usize,
    pub folders_or_data_stream_index: Either<u64, Vec<Folder>>,
    pub streams_unpack_sizes: Vec<u64>,
    pub folders_unpack_digests: Option<Vec<u32>>,
}
