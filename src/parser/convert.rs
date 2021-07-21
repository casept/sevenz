/// Convert a u64 to usize or error on failure.
macro_rules! to_usize_or_err {
( $( $x:expr ),+ ) => {
        {
            $(
                use core::convert::TryFrom;
                match usize::try_from($x) {
                       Ok(res) => res,
                       Err(e) => return Err(nom::Err::Error(crate::parser::err::SevenZParserError::new(crate::parser::err::SevenZParserErrorKind::ConversionFailure(crate::parser::err::SevenZConversionError::ToUsize(e))))),
               }
            )+
        }
    };
}
