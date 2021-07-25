use core::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyID {
    End = 0x00,
    Header = 0x01,
    ArchiveProperties = 0x02,
    AdditionalStreamsInfo = 0x03,
    MainStreamsInfo = 0x04,
    FilesInfo = 0x05,
    PackInfo = 0x06,
    UnPackInfo = 0x07,
    SubStreamsInfo = 0x08,
    Size = 0x09,
    CRC = 0x0A,
    Folder = 0x0B,
    CodersUnPackSize = 0x0C,
    NumUnPackStream = 0x0D,
    EmptyStream = 0x0E,
    EmptyFile = 0x0F,
    Anti = 0x10,
    Name = 0x11,
    CTime = 0x12,
    ATime = 0x13,
    MTime = 0x14,
    WinAttributes = 0x15,
    Comment = 0x16,
    EncodedHeader = 0x17,
    StartPos = 0x18,
    Dummy = 0x19,
}

impl TryFrom<u8> for PropertyID {
    type Error = ();
    fn try_from(b: u8) -> Result<Self, Self::Error> {
        use PropertyID::*;
        match b {
            0x00 => Ok(End),
            0x01 => Ok(Header),
            0x02 => Ok(ArchiveProperties),
            0x03 => Ok(AdditionalStreamsInfo),
            0x04 => Ok(MainStreamsInfo),
            0x05 => Ok(FilesInfo),
            0x06 => Ok(PackInfo),
            0x07 => Ok(UnPackInfo),
            0x08 => Ok(SubStreamsInfo),
            0x09 => Ok(Size),
            0x0A => Ok(CRC),
            0x0B => Ok(Folder),
            0x0C => Ok(CodersUnPackSize),
            0x0D => Ok(NumUnPackStream),
            0x0E => Ok(EmptyStream),
            0x0F => Ok(EmptyFile),
            0x10 => Ok(Anti),
            0x11 => Ok(Name),
            0x12 => Ok(CTime),
            0x13 => Ok(ATime),
            0x14 => Ok(MTime),
            0x15 => Ok(WinAttributes),
            0x16 => Ok(Comment),
            0x17 => Ok(EncodedHeader),
            0x18 => Ok(StartPos),
            0x19 => Ok(Dummy),
            _ => Err(()),
        }
    }
}
