//! Structures that make up 7zip archives.
//! These are "low-level", meaning that they're meant to
//! reflect how data is stored in the archive, not provide a friendly interface to it.

mod coders_info;
mod files_info;
mod header;
mod property_id;
mod streams_info;
pub use coders_info::*;
pub use files_info::*;
pub use header::*;
pub use property_id::*;
pub use streams_info::*;

use alloc::string::String;
use alloc::vec::Vec;
use bitvec::prelude::*;
use either::Either;

#[derive(Debug, Clone, PartialEq)]
pub struct PackInfo {
    pub pack_pos: u64,
    pub num_pack_streams: usize,
    pub sizes: Option<Vec<u64>>,
    pub crcs: Option<Vec<u32>>,
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
/// The top-level archive structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Archive {
    pub signature_header: SignatureHeader,
    pub header_or_packed_header: Either<Header, (PackedHeader, HeaderInfo)>,
}
