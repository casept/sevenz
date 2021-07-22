use super::*;

pub fn coder(input: &[u8]) -> SevenZResult<Coder> {
    fn is_complex(props: u8) -> bool {
        (props & 0b0000_1000) > 0
    }
    fn has_attrs(props: u8) -> bool {
        (props & 0b0000_0100) > 0
    }
    fn id_len(props: u8) -> usize {
        ((props & 0b1111_0000) >> 4) as usize
    }

    // TODO: Error for illegally set bit 7

    let (input, props) = context("coder properties", u8)(input)?;
    let (input, id) = context("coder ID", take(id_len(props)))(input)?;
    let id = Vec::from(id);

    let (input, complex) = cond(
        is_complex(props),
        context(
            "coder number of complex streams",
            map(
                pair(sevenz_uint64, sevenz_uint64),
                |(num_in_streams, num_out_streams)| CoderComplex {
                    num_in_streams,
                    num_out_streams,
                },
            ),
        ),
    )(input)?;

    let (input, attrs) = context(
        "coder attributes",
        cond(has_attrs(props), length_count(sevenz_uint64_as_usize, u8)),
    )(input)?;

    return Ok((input, Coder { complex, attrs, id }));
}

pub fn folder_coders(input: &[u8]) -> SevenZResult<Vec<Coder>> {
    let (input, coders_vec) = context(
        "folder_coders coders",
        length_count(
            context("folder_coders num_coders", sevenz_uint64_as_usize),
            context("folder_coders coders", coder),
        ),
    )(input)?;
    return Ok((input, coders_vec));
}

pub fn folder(input: &[u8]) -> SevenZResult<Folder> {
    let (input, coders_vec) = context("folder coders", folder_coders)(input)?;

    let num_out_streams_total: u64 = coders_vec
        .iter()
        .map(|x| match x.complex {
            Some(c) => c.num_out_streams,
            None => 1,
        })
        .sum();
    let num_out_streams_total: usize = to_usize_or_err!(num_out_streams_total);

    let num_bind_pairs = num_out_streams_total - 1;
    let (input, bind_pairs) = context(
        "folder bind_pairs",
        count(pair(sevenz_uint64, sevenz_uint64), num_bind_pairs),
    )(input)?;

    let num_in_streams_total: u64 = coders_vec
        .iter()
        .map(|x| match x.complex {
            Some(c) => c.num_in_streams,
            None => 1,
        })
        .sum();
    let num_in_streams_total: usize = to_usize_or_err!(num_in_streams_total);
    let num_packed_streams = num_in_streams_total - num_bind_pairs;

    // TODO: The spec says that it should be num_packed_streams > 1, but in that case we get a leftover byte.
    let (input, packed_streams_indices) = context(
        "folder packed_streams_indices",
        cond(
            num_packed_streams >= 1,
            count(sevenz_uint64, num_packed_streams),
        ),
    )(input)?;

    return Ok((
        input,
        Folder {
            coders: coders_vec,
            bind_pairs,
            packed_streams_indices,
        },
    ));
}

pub fn coders_info(input: &[u8]) -> SevenZResult<CodersInfo> {
    let (input, _) = context(
        "coders_info PropertyID::UnPackInfo",
        tag([PropertyID::UnPackInfo as u8]),
    )(input)?;
    let (input, _) = context(
        "coders_info PropertyID::Folder",
        tag([PropertyID::Folder as u8]),
    )(input)?;

    let (input, num_folders) = context("coders_info num_folders", sevenz_uint64_as_usize)(input)?;

    let (input, external) = context("coders_info external", bool_byte)(input)?;

    let (input, folders_or_data_stream_index) = either(
        external,
        context("coders_info data_stream_index", sevenz_uint64),
        context("coders_info folders", count(folder, num_folders)),
    )(input)?;

    let (input, _) = context(
        "coders_info PropertyID::CodersUnPackSize",
        tag([PropertyID::CodersUnPackSize as u8]),
    )(input)?;

    // Read output stream sizes of all folders
    let all_coders: Vec<Coder> = folders_or_data_stream_index
        .clone()
        .right()
        .unwrap()
        .iter()
        .map(|x| x.coders.clone())
        .flatten()
        .collect();
    let num_total_out_streams: u64 = all_coders
        .iter()
        .map(|x| match x.complex {
            Some(c) => c.num_out_streams,
            None => 1,
        })
        .sum();
    let num_total_out_streams = to_usize_or_err!(num_total_out_streams);

    let (input, streams_unpack_sizes) = context(
        "coders_info streams_unpack_sizes",
        count(sevenz_uint64, num_total_out_streams),
    )(input)?;

    let (input, folders_unpack_digests) = context(
        "coders_info unpack_digests",
        preceded_opt(tag([PropertyID::CRC as u8]), count(le_u32, num_folders)),
    )(input)?;

    let (input, _) = context("coders_info PropertyID::End", tag([PropertyID::End as u8]))(input)?;

    return Ok((
        input,
        CodersInfo {
            num_folders,
            folders_or_data_stream_index,
            streams_unpack_sizes,
            folders_unpack_digests,
        },
    ));
}
