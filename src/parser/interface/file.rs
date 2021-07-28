use core::convert::TryInto;

use super::*;

use alloc::string::String;
use alloc::vec;
use either::*;

/// More-or-less like `FilesInfo`, but for a single file and more high-level.
#[derive(Debug, Clone)]
pub struct File {
    // TODO: Figure out what exactly the u64 means and expose this as a `chrono` time instead
    pub ctime: Option<u64>,
    pub atime: Option<u64>,
    pub mtime: Option<u64>,
    // TODO: This should probably be exposed as a wide string instead, so that the user may decide what to do with it.
    pub name: String,
    pub stream: FileStreamInfo,
}

impl File {
    pub fn from_files_and_coders_info(fi: &FilesInfo, ci: &CodersInfo, index: usize) -> File {
        let ctimes = fi.get_property(FilesProperty::CTime(vec![]));
        let ctime = match ctimes {
            Some(ct) => match ct {
                FilesProperty::CTime(t) => match t[index] {
                    None => None,
                    Some(eith) => match eith {
                        Left(_) => panic!("External ctime not yet supported!"),
                        Right(t) => Some(t),
                    },
                },
                _ => panic!("Unexpected enum variant! This is a bug."),
            },
            None => None,
        };

        let atimes = fi.get_property(FilesProperty::ATime(vec![]));
        let atime = match atimes {
            Some(at) => match at {
                FilesProperty::CTime(t) => match t[index] {
                    None => None,
                    Some(eith) => match eith {
                        Left(_) => panic!("External atime not yet supported!"),
                        Right(t) => Some(t),
                    },
                },
                _ => panic!("Unexpected enum variant! This is a bug."),
            },
            None => None,
        };

        let mtimes = fi.get_property(FilesProperty::MTime(vec![]));
        let mtime = match mtimes {
            Some(mt) => match mt {
                FilesProperty::CTime(t) => match t[index] {
                    None => None,
                    Some(eith) => match eith {
                        Left(_) => panic!("External mtime not yet supported!"),
                        Right(t) => Some(t),
                    },
                },
                _ => panic!("Unexpected enum variant! This is a bug."),
            },
            None => None,
        };

        let names = fi.get_property(FilesProperty::Names(vec![]));
        let name = match names {
            Some(nm) => match nm {
                FilesProperty::Names(t) => match t[index].clone() {
                    Left(_) => panic!("External name not yet supported!"),
                    Right(t) => Some(t),
                },
                _ => panic!("Unexpected enum variant! This is a bug."),
            },
            None => None,
        };
        let name = name.unwrap();

        let stream = get_file_streams_info(ci)[index].clone();

        return File {
            ctime,
            atime,
            mtime,
            name,
            stream,
        };
    }

    /// Take a subslice of raw file data at the file's stream's starting offset from the given data buffer.
    ///
    /// Note that this data is not yet decoded.
    pub fn subslice_raw<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        let offset: usize = self.stream.offset.try_into().unwrap();
        let offset: usize = offset + START_HEADER_SIZE_BYTES;
        let size: usize = self.stream.size.try_into().unwrap();
        return &data[offset..offset + size];
    }
}
