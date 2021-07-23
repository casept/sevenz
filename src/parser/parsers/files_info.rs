use super::*;
use alloc::string::String;
use bitvec::prelude::*;
use either::*;
use widestring::*;

fn empty_stream(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context(
        "property_empty_stream PropertyID::EmptyStream",
        tag([PropertyID::EmptyStream as u8]),
    )(input)?;
    let (input, _size) = context("empty_stream size", sevenz_uint64)(input)?;

    let (input, bits) = context("property_empty_stream is_empty bits", |x| {
        take_bitvec(x, num_files)
    })(input)?;

    return Ok((input, FilesProperty::EmptyStream(bits)));
}

fn empty_file(input: &[u8], num_empty_streams: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context(
        "property_empty_file PropertyID::EmptyFile",
        tag([PropertyID::EmptyFile as u8]),
    )(input)?;
    let (input, _size) = context("empty_file size", sevenz_uint64)(input)?;

    let (input, bits) = context("property_empty_file is_empty bits", |x| {
        take_bitvec(x, num_empty_streams)
    })(input)?;

    return Ok((input, FilesProperty::EmptyFile(bits)));
}

fn anti(input: &[u8], num_empty_streams: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context("anti PropertyID::Anti", tag([PropertyID::Anti as u8]))(input)?;
    let (input, _size) = context("anti size", sevenz_uint64)(input)?;

    let (input, bits) = context("anti is_anti bits", |x| take_bitvec(x, num_empty_streams))(input)?;

    return Ok((input, FilesProperty::Anti(bits)));
}

fn time(input: &[u8], num_files: usize) -> SevenZResult<Vec<Option<FileTime>>> {
    let (input, _size) = context("time size", sevenz_uint64)(input)?;
    // Fill BitVec telling us which files have timestamps defined,
    // or fill it with `true` if all are defined.
    let (input, all_defined) = context("property_time all_defined", bool_byte)(input)?;
    let (input, defined): (&[u8], Option<BitVec>) = cond(
        !all_defined,
        context("property_time time_defined", |x| take_bitvec(x, num_files)),
    )(input)?;
    let defined = match defined {
        Some(d) => d,
        None => {
            let bits: Vec<bool> = core::iter::repeat(true).take(num_files).collect();
            let mut b = BitVec::new();
            b.extend_from_slice(&bits);
            b
        }
    };

    // TODO: Actually read externally-stored data (though maybe not here)
    let (input, external) = context("time external", bool_byte)(input)?;
    let (input, data_idx) = cond(external, context("time data_idx", le_u64))(input)?;
    match data_idx {
        Some(i) => {
            let all_external = core::iter::repeat(i)
                .take(num_files)
                .map::<Option<Either<u64, u64>>, _>(|x| Some(Left(x)))
                .collect();
            return Ok((input, all_external));
        }
        None => (),
    }

    // Read actual timestamps
    let (input, times): (&[u8], Vec<Option<u64>>) = many_cond_opt(le_u64, defined)(input)?;
    let ret: Vec<Option<Either<u64, u64>>> = times
        .iter()
        .map(|x| match x {
            Some(x) => Some(Right(*x)),
            None => None,
        })
        .collect();
    return Ok((input, ret));
}

fn ctime(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context("ctime PropertyID::CTime", tag([PropertyID::CTime as u8]))(input)?;
    let (input, ctime) = context("ctime time", |x| time(x, num_files))(input)?;
    return Ok((input, FilesProperty::CTime(ctime)));
}

fn atime(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context("atime PropertyID::ATime", tag([PropertyID::ATime as u8]))(input)?;
    let (input, atime) = context("atime time", |x| time(x, num_files))(input)?;
    return Ok((input, FilesProperty::ATime(atime)));
}

fn mtime(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context("mtime PropertyID::MTime", tag([PropertyID::MTime as u8]))(input)?;
    let (input, mtime) = context("mtime time", |x| time(x, num_files))(input)?;
    return Ok((input, FilesProperty::ATime(mtime)));
}

/// Parse a null-terminated string made of Windows-style UTF-16LE codepoints.
fn wchar_str(input: &[u8]) -> SevenZResult<String> {
    // Read until '\0' into temporary data buffer
    let (input, (mut data, _)) = context("wchar_str data", many_till(le_u16, tag([0, 0])))(input)?;
    data.push(0);
    // Convert
    let win_str = U16CStr::from_slice_with_nul(&data).unwrap();
    let res = match win_str.to_string() {
        Ok(s) => s,
        Err(_) => {
            return Err(nom::Err::Error(SevenZParserError::<&[u8]>::new(
                SevenZParserErrorKind::<&[u8]>::ConversionFailure(SevenZConversionError::ToString),
            )))
        }
    };
    return Ok((input, res));
}

fn names(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context("names PropertyID::Name", tag([PropertyID::Name as u8]))(input)?;
    let (input, _size) = context("names size", sevenz_uint64)(input)?;
    let (input, external) = context("names external", bool_byte)(input)?;

    // TODO: Actually support external data
    let (input, data_idx) = cond(external, context("names data_idx", le_u64))(input)?;
    match data_idx {
        Some(i) => {
            let all_external = core::iter::repeat(i)
                .take(num_files)
                .map::<Either<u64, String>, _>(|x| Left(x))
                .collect();
            return Ok((input, FilesProperty::Names(all_external)));
        }
        None => (),
    };

    // Read actual names
    let (input, names) = context("names names", count(wchar_str, num_files))(input)?;
    let names = names.iter().map(|x| Right(x.clone())).collect();
    return Ok((input, FilesProperty::Names(names)));
}

fn attrs(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context(
        "attrs PropertyID::WinAttributes",
        tag([PropertyID::WinAttributes as u8]),
    )(input)?;
    let (input, _size) = context("attrs size", sevenz_uint64)(input)?;
    // Fill BitVec telling us which files have attrs defined,
    // or fill it with `true` if all are defined.
    let (input, all_defined) = context("attrs all_defined", bool_byte)(input)?;
    let (input, defined): (&[u8], Option<BitVec>) = cond(
        !all_defined,
        context("attrs attrs_defined", |x| take_bitvec(x, num_files)),
    )(input)?;
    let defined = match defined {
        Some(d) => d,
        None => {
            let bits: Vec<bool> = core::iter::repeat(true).take(num_files).collect();
            let mut b = BitVec::new();
            b.extend_from_slice(&bits);
            b
        }
    };

    // TODO: Actually read externally-stored data (though maybe not here)
    let (input, external) = context("attrs external", bool_byte)(input)?;
    let (input, data_idx) = cond(external, context("attrs data_idx", le_u64))(input)?;
    match data_idx {
        Some(i) => {
            let all_external = core::iter::repeat(i)
                .take(num_files)
                .map::<Option<Either<u64, u32>>, _>(|x| Some(Left(x)))
                .collect();
            return Ok((input, FilesProperty::Attributes(all_external)));
        }
        None => (),
    }

    // Read actual attrs
    let (input, attrs): (&[u8], Vec<Option<u32>>) = many_cond_opt(le_u32, defined)(input)?;
    let ret: Vec<Option<Either<u64, u32>>> = attrs
        .iter()
        .map(|x| match x {
            Some(x) => Some(Right(*x)),
            None => None,
        })
        .collect();
    return Ok((input, FilesProperty::Attributes(ret)));
}

/// Reads and ignores a dummy property.
/// These are not documented in 7zFormat.txt, but according to https://sourceforge.net/p/sevenzip/discussion/45797/thread/0f3f75c9/
/// are used for ensuring alignment as an optimization technique.
/// Always returns `None` to make property() easier to implement.
fn dummy(input: &[u8]) -> SevenZResult<Option<FilesProperty>> {
    let (input, _) = context("dummy PropertyID::Dummy", tag([PropertyID::Dummy as u8]))(input)?;
    let (input, size) = context("dummy size", sevenz_uint64)(input)?;
    let size = to_usize_or_err!(size);
    let (input, data) = context("dummy data", count(u8, size))(input)?;
    for d in data {
        if d != 0 {
            return Err(nom::Err::Error(SevenZParserError::<&[u8]>::new(
                SevenZParserErrorKind::<&[u8]>::DummyNotAllZeroes,
            )));
        }
    }
    return Ok((input, None));
}

/// Reads the next property, whatever it may be.
/// Returns `None` if a dummy was encountered.
fn property(
    input: &[u8],
    num_files: usize,
    num_empty_streams: usize,
) -> SevenZResult<Option<FilesProperty>> {
    let (input, prop) = context(
        "property",
        alt((
            wrap_some(|x| empty_stream(x, num_files)),
            wrap_some(|x| empty_file(x, num_empty_streams)),
            wrap_some(|x| anti(x, num_empty_streams)),
            wrap_some(|x| ctime(x, num_files)),
            wrap_some(|x| atime(x, num_files)),
            wrap_some(|x| mtime(x, num_files)),
            wrap_some(|x| names(x, num_files)),
            wrap_some(|x| attrs(x, num_files)),
            dummy,
        )),
    )(input)?;
    return Ok((input, prop));
}

pub fn files_info(input: &[u8], num_empty_streams: usize) -> SevenZResult<FilesInfo> {
    let (input, _) = context(
        "files_info PropertyID::FilesInfo",
        tag([PropertyID::FilesInfo as u8]),
    )(input)?;

    let (input, num_files) = context("files_info num_files", sevenz_uint64_as_usize)(input)?;

    let (input, (files_properties, _)) = context(
        "files_info files_properties",
        many_till(
            |x| property(x, num_files, num_empty_streams),
            tag([PropertyID::End as u8]),
        ),
    )(input)?;
    let files_properties = files_properties
        .iter()
        .filter(|x| x.is_some())
        .map(|x| x.clone().unwrap())
        .collect();

    return Ok((
        input,
        FilesInfo {
            num_files,
            properties: files_properties,
        },
    ));
}
