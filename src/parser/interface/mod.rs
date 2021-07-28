//! This module contains a "high-level" interface to
//! the "low-level" types in `types`.
//!
//! This interface aims to make working with the archive easier than the raw data structures,
//! which are often idiosyncratic to save space in the archive.

mod archive;
mod file;
mod streams;
pub use super::err::SevenZParserError;
pub use archive::*;
pub use file::*;
pub use streams::*;

use super::types::*;
use super::*;

use either::*;

/// The entry point into the `parser` module.
/// Takes a byte slice, returns the parsed archive therein.
pub fn parse(input: &[u8]) -> Result<HighLevelArchive, SevenZParserError<&[u8]>> {
    let archive = match parsers::archive(input) {
        Ok((_, archive)) => archive,
        Err(e) => match e {
		nom::Err::Incomplete(_) => panic!("Parser reported incomplete. Before streaming support is implemented, this should never happen."),
		nom::Err::Error(e) => return Err(e),
		nom::Err::Failure(e) => return Err(e),
	},
    };
    return Ok(HighLevelArchive::from_low_level_archive(&archive));
}
