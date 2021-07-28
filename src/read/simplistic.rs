//! This module provides a "simplistic" API for reading archives.
//!
//! It trades off precise control for ease of use.

use super::err::Error;
use super::iter::ArchiveIterator;

use crate::parser::parse;

use alloc::string::String;
use alloc::vec::Vec;

/// Extract the file with the given `name` into a data buffer.
///
/// This involves parsing the entire archive and
/// iterating over all file descriptors for each file extracted, so it's not very efficient.
pub fn extract_file<'a>(name: &str, archive_data: &'a [u8]) -> Result<Vec<u8>, Error<'a>> {
    let ar = parse(archive_data)?;
    let files = ArchiveIterator::new(&ar);
    let file = match files.filter(|x| x.name() == name).next() {
        Some(f) => f,
        None => return Err(Error::NoSuchFileName(String::from(name))),
    };
    let contents = match file.extract_contents_vec() {
        Ok(contents) => contents,
        Err(e) => {
            // FIXME: Bubble up the error instead (requires decoupling error's lifetime from ar's lifetime)
            panic!("Failed to extract: {:?}", e);
        }
    };
    return Ok(contents);
}
