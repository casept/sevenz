//! This module implements dispatching file contents to decoders.

use alloc::vec::Vec;
use core::convert::TryFrom;

use crate::codec::*;
use crate::parser::FileStreamInfo;

/// Handle dispatching data to the appropriate decoder to extract the file.
///
/// Returns a fully decoded byte vector.
pub fn decode_file<'a>(data: &[u8], stream: &FileStreamInfo) -> Result<Vec<u8>, CodecError> {
    if stream.coder.complex.is_some() {
        panic!("Decoding using complex coders not yet supported!");
    }

    let codec_kind = Codecs::try_from(stream.coder.id.clone())?;
    match codec_kind {
        Codecs::Copy => {
            let copy_codec = Copy::new();
            return Ok(copy_codec.decode(data));
        }
    }
}
