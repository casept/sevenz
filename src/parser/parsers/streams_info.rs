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
        preceded_opt(
            |x| tag_property_id(x, PropertyID::Size),
            count(sevenz_uint64, num_pack_streams),
        ),
    )(input)?;
    let (input, crcs) = context(
        "pack_info crcs",
        preceded_opt(
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
        "coders_info num_unpack_streams_in_folders",
        preceded_opt(
            tag([PropertyID::NumUnPackStream as u8]),
            count(sevenz_uint64, num_folders),
        ),
    )(input)?;

    let (input, unpack_sizes) = context(
        "coders_info unpack_sizes",
        preceded_opt(
            tag([PropertyID::Size as u8]),
            count(sevenz_uint64, {
                // FIXME: Don't panic
                //let total_streams: u64 = num_unpack_streams_in_folders.unwrap().iter().sum();
                let total_streams: u64 = 1;
                to_usize_or_err!(total_streams)
            }),
        ),
    )(input)?;

    let (input, unknown_crcs) = context(
        "coders_info unknown_crcs",
        preceded_opt(
            tag([PropertyID::CRC as u8]),
            count(sevenz_uint64, num_unknown_crcs),
        ),
    )(input)?;

    let (input, _) = context(
        "substreams_info PropertyID::End",
        tag([PropertyID::End as u8]),
    )(input)?;

    return Ok((input, SubStreamsInfo {}));
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
        opt(|x| substreams_info(x, 1, 2)),
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
