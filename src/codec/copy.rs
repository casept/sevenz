use super::Codec;

use alloc::vec::Vec;

/// The trivial codec.
/// Simply shuffles bytes it gets back out.
pub struct Copy {}

impl Copy {
    /// Creates a new `Copy` codec.
    /// Because this codec doesn't really need construction, this ctor is only implemented for the sake of uniformity.
    pub fn new() -> Copy {
        return Copy {};
    }
}

impl Codec for Copy {
    fn decode(&self, data: &[u8]) -> Vec<u8> {
        return Vec::from(data);
    }
}
