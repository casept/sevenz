//! This module implements an interface for reading 7zip archives.

mod decode;
mod err;
mod iter;
mod simplistic;

use crate::parser::File;
use crate::parser::HighLevelArchive;

pub use err::*;
pub use iter::*;
pub use simplistic::*;

use alloc::string::String;
use alloc::vec::Vec;

/// The handle for a file contained within an archive.
///
/// Because this struct does not contain the actual file data but only the metadata
/// required to retrieve it from the archive,
/// it can't live independently of it's underlying archive.
pub struct FileHandle<'a> {
    pub(crate) underlying: File,
    // TODO: Expose ctime, mtime, atime, compressed size, uncompressed size and other useful attributes.
    /// The entire archive's unprocessed data.
    pub(crate) archive_raw: &'a [u8],
}

impl<'a> FileHandle<'a> {
    /// Create a new handle based on the given archive and file index.
    fn new(ar: &'a HighLevelArchive, archive_raw: &'a [u8], index: usize) -> FileHandle<'a> {
        let underlying = ar.files[index].clone();
        return FileHandle {
            underlying,
            archive_raw,
        };
    }

    /// Extract the file's contents into a vector.
    ///
    /// # Warning
    ///
    /// Note that this means the entire file will be loaded into RAM!
    ///
    /// It's recommended that you only use this method if you have checked that the file will fit using `FileHandle::size()`.
    ///
    /// Otherwise, you program will likely run out of memory.
    pub fn extract_contents_vec(&self) -> Result<Vec<u8>, err::Error<'a>> {
        let contents_packed = self.underlying.subslice_raw(self.archive_raw);
        let contents_unpacked = decode::decode_file(contents_packed, &self.underlying.stream)?;
        return Ok(contents_unpacked);
    }

    /// Get the name of this file.
    pub fn name(&self) -> String {
        return self.underlying.name.clone();
    }

    // TODO: Implement method to get contents into a `std::io::Write` or preferably a `no_std`-friendly equivalent.
}
