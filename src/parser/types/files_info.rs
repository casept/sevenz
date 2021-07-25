use super::*;

/// Left: external data index, right: time
pub type FileTime = Either<u64, u64>;

/// Left: external data index, right: name
pub type FileName = Either<u64, String>;

/// Left: External data index, right: attrs
pub type FileAttr = Either<u64, u32>;

#[derive(Debug, Clone, PartialEq)]
pub enum FilesProperty {
    EmptyStream(BitVec),
    EmptyFile(BitVec),
    Anti(BitVec),
    CTime(Vec<Option<FileTime>>),
    ATime(Vec<Option<FileTime>>),
    MTime(Vec<Option<FileTime>>),
    Names(Vec<FileName>),
    Attributes(Vec<Option<FileAttr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilesInfo {
    pub num_files: usize,
    pub properties: Vec<FilesProperty>,
}

impl FilesInfo {
    pub fn get_property(&self, kind: FilesProperty) -> Option<FilesProperty> {
        let props: Vec<FilesProperty> = self
            .properties
            .iter()
            .map(|x| *x)
            .filter(|x| match x {
                kind => true,
                _ => false,
            })
            .take(1)
            .collect();
        let prop = if props.len() > 0 {
            Some(props[0])
        } else {
            None
        };
        return prop;
    }
}
