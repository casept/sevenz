//! This module provides the top-level error type for this crate.

use crate::codec::CodecError;
use crate::parser::SevenZParserError;

use alloc::string::String;
use core::convert::From;

/// The top-level error type for this crate.
///
/// The underlying data buffer must live at least as long as the error,
/// because the parser errors contain subslices where a parsing issue occurred.
#[derive(Debug, Clone)]
pub enum Error<'a> {
    Parser(SevenZParserError<&'a [u8]>),
    NoSuchFileName(String),
    CodecFailed(CodecError),
}

impl<'a> From<SevenZParserError<&'a [u8]>> for Error<'a> {
    fn from(e: SevenZParserError<&'a [u8]>) -> Self {
        return Error::Parser(e);
    }
}

impl<'a> From<CodecError> for Error<'a> {
    fn from(e: CodecError) -> Self {
        return Error::CodecFailed(e);
    }
}
