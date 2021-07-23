use alloc::vec::Vec;
use core::convert::{From, TryFrom};
use nom::error::*;

/// Error type for failed conversions.
#[derive(Debug, Clone, PartialEq)]
pub enum SevenZConversionError {
    ToUsize(<usize as TryFrom<u64>>::Error),
    // Actual FromUtf16Error not saved because it would ruin Clone and PartialEq impl
    ToString,
}

/// The types of errors that may be returned by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum SevenZParserErrorKind<I> {
    Nom(I, nom::error::ErrorKind),
    // Crc(expected, got)
    Crc(u32, u32),
    // InvalidPropertyID(id)
    InvalidPropertyID(u8),
    ConversionFailure(SevenZConversionError),
    // InvalidBooleanByte(value)
    InvalidBooleanByte(u8),
    FilesEmptyFileBeforeFilesEmptyStream,
    FilesAntiBeforeFilesEmptyStream,
    DummyNotAllZeroes,
    CouldNotDetermineNumFolders,
    CouldNotDetermineNumUnpackStreams,
}

/// The error type returned by all parsers.
#[derive(Debug, Clone)]
pub struct SevenZParserError<I> {
    /// What kind of error this is
    pub kind: SevenZParserErrorKind<I>,
    /// All the context we have accumulated from previous errors.
    pub ctx: Vec<(I, &'static str)>,
}

impl<I> ParseError<I> for SevenZParserError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        return SevenZParserError::new(SevenZParserErrorKind::Nom(input, kind));
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> SevenZParserError<I> {
    /// Converts a builtin nom error to this error type.
    pub fn from_err(e: nom::Err<nom::error::Error<I>>) -> Self {
        use nom::Err::*;
        match e {
            Incomplete(_) => panic!("Encountered incomplete, not sure what to do"),
            Error(inner) => return SevenZParserError::from_error_kind(inner.input, inner.code),
            Failure(inner) => return SevenZParserError::from_error_kind(inner.input, inner.code),
        }
    }

    /// Creates a new error.
    pub fn new(kind: SevenZParserErrorKind<I>) -> Self {
        return SevenZParserError {
            kind,
            ctx: Vec::new(),
        };
    }
}

impl<I> ContextError<I> for SevenZParserError<I> {
    fn add_context(_input: I, _ctx: &'static str, mut other: Self) -> Self {
        other.ctx.push((_input, _ctx));
        return other;
    }
}

impl<I> From<SevenZConversionError> for SevenZParserError<I> {
    fn from(e: SevenZConversionError) -> Self {
        SevenZParserError::<I>::new(SevenZParserErrorKind::ConversionFailure(e))
    }
}
