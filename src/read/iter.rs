use super::FileHandle;
use crate::parser::HighLevelArchive;

use core::iter::Iterator;

/// Iterates over each file in the archive.
/// Actual extraction occurs only once the file's contents are requested.
///
/// Underlying `Archive` must live at least as long.
#[derive(Debug, Clone)]
pub struct ArchiveIterator<'a> {
    ar: &'a HighLevelArchive,
}

impl<'a> ArchiveIterator<'a> {
    /// Create a new iterator over the given `Archive`.
    pub fn new(ar: &'a HighLevelArchive) -> ArchiveIterator<'a> {
        ArchiveIterator { ar }
    }
}

impl<'a> Iterator for ArchiveIterator<'a> {
    type Item = FileHandle<'a>;
    fn next(&mut self) -> Option<FileHandle<'a>> {
        // FIXME: Implement
        None
    }
}
