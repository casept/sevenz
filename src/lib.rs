#![forbid(unsafe_code)]
//! A crate for interacting with 7zip files.
//! Archives can currently only be read, but support for writing would be nice.

#![no_std]
#![allow(clippy::needless_return)]

extern crate alloc;

mod parser;
pub mod read;
