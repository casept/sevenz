use super::*;

pub const START_HEADER_SIZE_BYTES: usize = 8 + 8 + 4;

#[derive(Debug, Clone, PartialEq)]
pub struct StartHeader {
    pub next_header_offset: u64,
    pub next_header_size: u64,
    pub next_header_crc: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchiveVersion {
    pub major: u8,
    pub minor: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignatureHeader {
    pub archive_version: ArchiveVersion,
    pub start_header_crc: u32,
    pub start_header: StartHeader,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub archive_properties: Option<ArchiveProperties>,
    pub additional_streams: Option<StreamsInfo>,
    pub main_streams: Option<StreamsInfo>,
    pub files: Option<FilesInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackedHeader {}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderInfo {}
