//! Structures that make up 7zip archives

use alloc::string::String;
use alloc::vec::Vec;
use bitvec::prelude::*;
use core::convert::TryFrom;
use either::Either;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyID {
    End = 0x00,
    Header = 0x01,
    ArchiveProperties = 0x02,
    AdditionalStreamsInfo = 0x03,
    MainStreamsInfo = 0x04,
    FilesInfo = 0x05,
    PackInfo = 0x06,
    UnPackInfo = 0x07,
    SubStreamsInfo = 0x08,
    Size = 0x09,
    CRC = 0x0A,
    Folder = 0x0B,
    CodersUnPackSize = 0x0C,
    NumUnPackStream = 0x0D,
    EmptyStream = 0x0E,
    EmptyFile = 0x0F,
    Anti = 0x10,
    Name = 0x11,
    CTime = 0x12,
    ATime = 0x13,
    MTime = 0x14,
    WinAttributes = 0x15,
    Comment = 0x16,
    EncodedHeader = 0x17,
    StartPos = 0x18,
    Dummy = 0x19,
}

impl TryFrom<u8> for PropertyID {
    type Error = ();
    fn try_from(b: u8) -> Result<Self, Self::Error> {
        use PropertyID::*;
        match b {
            0x00 => Ok(End),
            0x01 => Ok(Header),
            0x02 => Ok(ArchiveProperties),
            0x03 => Ok(AdditionalStreamsInfo),
            0x04 => Ok(MainStreamsInfo),
            0x05 => Ok(FilesInfo),
            0x06 => Ok(PackInfo),
            0x07 => Ok(UnPackInfo),
            0x08 => Ok(SubStreamsInfo),
            0x09 => Ok(Size),
            0x0A => Ok(CRC),
            0x0B => Ok(Folder),
            0x0C => Ok(CodersUnPackSize),
            0x0D => Ok(NumUnPackStream),
            0x0E => Ok(EmptyStream),
            0x0F => Ok(EmptyFile),
            0x10 => Ok(Anti),
            0x11 => Ok(Name),
            0x12 => Ok(CTime),
            0x13 => Ok(ATime),
            0x14 => Ok(MTime),
            0x15 => Ok(WinAttributes),
            0x16 => Ok(Comment),
            0x17 => Ok(EncodedHeader),
            0x18 => Ok(StartPos),
            0x19 => Ok(Dummy),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackInfo {
    pub pack_pos: u64,
    pub num_pack_streams: usize,
    pub sizes: Option<Vec<u64>>,
    pub crcs: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchiveVersion {
    pub major: u8,
    pub minor: u8,
}

pub const START_HEADER_SIZE_BYTES: usize = 8 + 8 + 4;

#[derive(Debug, Clone, PartialEq)]
pub struct StartHeader {
    pub next_header_offset: u64,
    pub next_header_size: u64,
    pub next_header_crc: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignatureHeader {
    pub archive_version: ArchiveVersion,
    pub start_header_crc: u32,
    pub start_header: StartHeader,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PackedStreams {}
#[derive(Debug, Clone, PartialEq)]
pub struct PackedStreamsForHeaders {}

#[derive(Debug, Clone, PartialEq)]
pub struct Packed {}

#[derive(Debug, Clone, PartialEq)]
pub struct ArchiveProperties {
    // Would be nice to have property data also be 0-copy, but that'd
    // lead to some messy lifetimes.
    pub property_data: Vec<(PropertyID, Vec<u8>)>,
}

/// Left: external data index, right: time
pub type FileTime = Either<u64, u64>;

/// Left: external data index, right: name
pub type FileName = Either<u64, String>;

/// Left: External data index, right: attrs
pub type FileAttr = Either<u64, u32>;

#[derive(Debug, Clone, PartialEq)]
pub enum FilesProperty {
    EmptyStream(BitVec),
    EmptyFile(BitVec),
    Anti(BitVec),
    CTime(Vec<Option<FileTime>>),
    ATime(Vec<Option<FileTime>>),
    MTime(Vec<Option<FileTime>>),
    Names(Vec<FileName>),
    Attributes(Vec<Option<FileAttr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilesInfo {
    pub num_files: usize,
    pub properties: Vec<FilesProperty>,
}

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
    // TODO: These should go into their respective folders
    pub streams_unpack_sizes: Vec<u64>,
    pub folders_unpack_digests: Option<Vec<u32>>,
}

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

/// The top-level archive structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Archive {
    pub signature_header: SignatureHeader,
    pub packed_streams: Option<Vec<PackedStreams>>,
    pub packed_streams_for_headers: Option<Vec<PackedStreamsForHeaders>>,
    pub header_or_packed_header: Either<Header, (PackedHeader, HeaderInfo)>,
}
