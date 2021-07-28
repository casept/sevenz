use super::*;

pub fn pack_info(input: &[u8]) -> SevenZResult<PackInfo> {
    let (input, _) = context(
        "pack_info PropertyID::PackInfo",
        tag([PropertyID::PackInfo as u8]),
    )(input)?;
    let (input, pack_pos) = context("pack_info pack_pos", sevenz_uint64)(input)?;
    let (input, num_pack_streams) =
        context("pack_info num_pack_streams", sevenz_uint64_as_usize)(input)?;

    // TODO: The spec is not exactly clear about the circumstances under which these 2 are optional.
    // TODO: For now, let's just assume that they're optional when their markers are present and vice versa.
    let (input, sizes) = context(
        "pack_info sizes",
        preceded_opt_lazy(
            |x| tag_property_id(x, PropertyID::Size),
            count(sevenz_uint64, num_pack_streams),
        ),
    )(input)?;
    let (input, crcs) = context(
        "pack_info crcs",
        preceded_opt_lazy(
            |x| tag_property_id(x, PropertyID::CRC),
            count(le_u32, num_pack_streams),
        ),
    )(input)?;

    let (input, _) = context("pack_info PropertyID::End", tag([PropertyID::End as u8]))(input)?;
    return Ok((
        input,
        PackInfo {
            pack_pos,
            num_pack_streams,
            sizes: sizes,
            crcs: crcs,
        },
    ));
}

pub fn substreams_info(
    input: &[u8],
    num_folders: usize,
    num_unknown_crcs: usize,
) -> SevenZResult<SubStreamsInfo> {
    let (input, _) = context(
        "substreams_info PropertyID::SubStreamsInfo",
        tag([PropertyID::SubStreamsInfo as u8]),
    )(input)?;

    let (input, num_unpack_streams_in_folders) = context(
        "substreams_info num_unpack_streams_in_folders",
        preceded_opt_lazy(
            tag([PropertyID::NumUnPackStream as u8]),
            count(sevenz_uint64, num_folders),
        ),
    )(input)?;

    // Fail if we'd expect a size block next, but don't know how long it'd be
    let will_have_unpack_sizes =
        tag::<[u8; 1], &[u8], SevenZParserError<&[u8]>>([PropertyID::Size as u8])(input).is_ok();
    let total_streams = to_usize_or_err!(if will_have_unpack_sizes {
        let num: Vec<u64> = if num_unpack_streams_in_folders.clone().is_some() {
            num_unpack_streams_in_folders.clone().unwrap()
        } else {
            return Err(nom::Err::Failure(SevenZParserError::new(
                SevenZParserErrorKind::CouldNotDetermineNumUnpackStreams,
            )));
        };
        let v: u64 = num.iter().sum();
        v
    } else {
        0
    });

    let (input, unpack_sizes) = context(
        "substreams_info unpack_sizes",
        preceded_opt_lazy(
            tag([PropertyID::Size as u8]),
            count(sevenz_uint64, total_streams),
        ),
    )(input)?;

    let (input, unknown_digests) = context(
        "substreams_info unknown_crcs",
        preceded_opt_lazy(
            tag([PropertyID::CRC as u8]),
            count(le_u32, num_unknown_crcs),
        ),
    )(input)?;

    let (input, _) = context(
        "substreams_info PropertyID::End",
        tag([PropertyID::End as u8]),
    )(input)?;

    return Ok((
        input,
        SubStreamsInfo {
            num_unpack_streams_in_folders,
            unpack_sizes,
            unknown_digests,
        },
    ));
}

/// Read a Streams Info structure.
/// As the structure appears multiple times, it may not be complete each time.
/// Therefore, if this isn't the first time reading this structure you have to pass in `num_folders`,
/// because this info may not be available.
pub fn streams_info(input: &[u8], num_folders: Option<usize>) -> SevenZResult<StreamsInfo> {
    let (input, pack_info_data) = context("streams_info pack_info", opt(pack_info))(input)?;
    let (input, coders_info_data) = context("streams_info coders_info", opt(coders_info))(input)?;
    // Use given value or retrieve num_folders
    let num_folders = if num_folders.is_some() {
        num_folders.unwrap()
    } else {
        match coders_info_data.clone() {
            Some(c) => c.num_folders,
            None => {
                return Err(nom::Err::Failure(SevenZParserError::new(
                    SevenZParserErrorKind::CouldNotDetermineNumFolders,
                )))
            }
        }
    };

    // FIXME: Don't hardcode num_unknown_crcs
    let (input, substreams_info_data) = context(
        "streams_info substreams_info",
        opt(|x| substreams_info(x, num_folders, 3)),
    )(input)?;
    let (input, _) = context("streams_info PropertyID::End", tag([PropertyID::End as u8]))(input)?;

    // TODO:
    return Ok((
        input,
        StreamsInfo {
            pack_info: pack_info_data,
            coders_info: coders_info_data,
            substreams_info: substreams_info_data,
        },
    ));
}
