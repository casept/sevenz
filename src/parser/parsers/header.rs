use super::*;

pub fn archive_property(input: &[u8]) -> SevenZResult<(PropertyID, &[u8])> {
    let (input, prop_type) = context("archive_property prop_type", property_id)(input)?;
    let (input, len) = context("archive_property len", sevenz_uint64_as_usize)(input)?;
    let (input, prop_data) = context("archive_property prop_data", take(len))(input)?;
    return Ok((input, (prop_type, prop_data)));
}

pub fn archive_properties(input: &[u8]) -> SevenZResult<ArchiveProperties> {
    let (input, _) = context(
        "archive_properties PropertyID::ArchiveProperties",
        tag([PropertyID::ArchiveProperties as u8]),
    )(input)?;
    let mut props: Vec<(PropertyID, Vec<u8>)> = vec![];
    loop {
        let (input, (prop_id, prop_data)) = archive_property(input)?;
        if prop_id == PropertyID::End {
            return Ok((
                input,
                ArchiveProperties {
                    property_data: props,
                },
            ));
        }

        props.push((prop_id, Vec::from(prop_data)));
    }
}

pub fn header(input: &[u8]) -> SevenZResult<Header> {
    let (input, _) = context("header PropertyID::Header", tag([PropertyID::Header as u8]))(input)?;

    let (input, archive_properties) =
        context("header archive_properties", opt(archive_properties))(input)?;

    let (input, have_asi) = context(
        "header PropertyID::AdditionalStreamsInfo",
        opt(tag([PropertyID::AdditionalStreamsInfo as u8])),
    )(input)?;
    let (input, additional_streams) = context(
        "header additional_streams",
        cond(have_asi.is_some(), |x| streams_info(x, None)),
    )(input)?;

    let (input, have_msi) = context(
        "header PropertyID::MainStreamsInfo",
        opt(tag([PropertyID::MainStreamsInfo as u8])),
    )(input)?;
    // Try to retrieve num_folders
    let num_folders = if additional_streams.is_some()
        && additional_streams.clone().unwrap().coders_info.is_some()
    {
        Some(
            additional_streams
                .clone()
                .unwrap()
                .coders_info
                .unwrap()
                .num_folders,
        )
    } else {
        None
    };

    let (input, main_streams) = context(
        "header main_streams",
        cond(have_msi.is_some(), |x| streams_info(x, num_folders)),
    )(input)?;

    // FIXME: Pass proper num_empty streams
    let (input, files) = context("header files_info", opt(|x| files_info(x, 0)))(input)?;
    let (input, _) = context("header PropertyID::End", tag([PropertyID::End as u8]))(input)?;

    return Ok((
        input,
        Header {
            archive_properties,
            additional_streams,
            main_streams,
            files,
        },
    ));
}

/*
pub fn archive(input: &[u8]) -> SevenZResult<Archive> {
    let (input, signature_header) = to_err!(signature_header(input));
    return Ok((
        input,
        Archive {
            signature_header,
            packed_streams,
            packed_streams_for_headers,
            header_or_packed_header,
        },
    ));
}
*/
