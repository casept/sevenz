//! This module contains decoders (and, eventually, encoders)
//! for stream formats documented in 7zip's methods.txt.

mod copy;
pub use copy::*;

use alloc::vec::Vec;
use core::convert::TryFrom;

/// The main interface trait for other code to use.
///
/// All codecs must implement it.
pub trait Codec {
    /// Take the given data buffer and decode it.
    ///
    /// Any required additional information that the codec can't deduce from the data stream
    /// must be passed via the codec-specific constructor.
    ///
    /// The `&self` parameter is currently immutable, but may be made mutable in future
    /// if a coded is determined to require it.
    ///
    /// Also, the simplistic u8 data buffers will probably be swapped out in future to enable streaming.
    fn decode(&self, data: &[u8]) -> Vec<u8>;
}

/// All currently supported codecs.
pub enum Codecs {
    /// As the name implies, simply copies the data byte-for-byte.
    Copy,
}

impl TryFrom<Vec<u8>> for Codecs {
    type Error = CodecError;
    /// Parse a codec ID.
    /// It may be 1 to 4 bytes long.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // 00 - Copy
        match value.as_slice() {
            [0] => return Ok(Codecs::Copy),
            _ => return Err(CodecError::InvalidCodecID(value)),
        }
    }
}

/// The top-level codec error type.
#[derive(Debug, Clone)]
pub enum CodecError {
    /// The archive contained an invalid codec ID.
    InvalidCodecID(Vec<u8>),
}
