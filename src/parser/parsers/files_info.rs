use super::*;
use alloc::string::String;
use bitvec::prelude::*;
use either::*;
use widestring::*;

fn property_empty_stream(input: &[u8], num_files: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context(
        "property_empty_stream PropertyID::EmptyStream",
        tag([PropertyID::EmptyStream as u8]),
    )(input)?;

    let (input, bits) = context("property_empty_stream is_empty bits", |x| {
        take_bitvec(x, num_files)
    })(input)?;

    return Ok((input, FilesProperty::EmptyStream(bits)));
}

fn property_empty_file(input: &[u8], num_empty_streams: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context(
        "property_empty_file PropertyID::EmptyFile",
        tag([PropertyID::EmptyFile as u8]),
    )(input)?;

    let (input, bits) = context("property_empty_file is_empty bits", |x| {
        take_bitvec(x, num_empty_streams)
    })(input)?;

    return Ok((input, FilesProperty::EmptyFile(bits)));
}

fn property_anti(input: &[u8], num_empty_streams: usize) -> SevenZResult<FilesProperty> {
    let (input, _) = context(
        "property_empty_file PropertyID::Anti",
        tag([PropertyID::Anti as u8]),
    )(input)?;

    let (input, bits) = context("property_anti is_anti bits", |x| {
        take_bitvec(x, num_empty_streams)
    })(input)?;

    return Ok((input, FilesProperty::Anti(bits)));
}

fn time(input: &[u8], num_files: usize) -> SevenZResult<Vec<Option<FileTime>>> {
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
    let (input, external) = context("property_time external", bool_byte)(input)?;
    let (input, data_idx) = cond(external, context("property_time data_idx", le_u64))(input)?;
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
    let (input, (mut data, _)) = context("wchar_str data", many_till(le_u16, tag([0])))(input)?;
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

// TODO: attrs

fn property(
    input: &[u8],
    num_files: usize,
    num_empty_streams: usize,
) -> SevenZResult<FilesProperty> {
    let (input, prop) = context(
        "files_property",
        alt((
            |x| property_empty_stream(x, num_files),
            |x| property_empty_file(x, num_empty_streams),
            |x| property_anti(x, num_empty_streams),
            |x| ctime(x, num_files),
            |x| atime(x, num_files),
            |x| mtime(x, num_files),
            |x| names(x, num_files),
        )),
    )(input)?;
    return Ok((input, prop));
}

pub fn files_info(input: &[u8]) -> SevenZResult<FilesInfo> {
    let (input, _) = context(
        "files_info PropertyID::FilesInfo",
        tag([PropertyID::FilesInfo as u8]),
    )(input)?;

    let (input, num_files) = context("files_info num_files", sevenz_uint64_as_usize)(input)?;
    let (input, (files_properties, _)) = context(
        "files_info files_properties",
        // FIXME: Pass correct value to num_empty_streams
        many_till(|x| property(x, num_files, 0), tag([PropertyID::End as u8])),
    )(input)?;

    return Ok((
        input,
        FilesInfo {
            num_files,
            properties: files_properties,
        },
    ));
}
