use either::*;
use nom::error::ParseError;
use nom::IResult;

/// Runs the second parser and returns it's output/error if the first parser succeeds.
/// Doesn't run the second parser and returns Ok((input, None)) if the first parser fails.
pub fn preceded_opt<I, O1, O2, E: ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, Option<O2>, E>
where
    F: nom::Parser<I, O1, E>,
    G: nom::Parser<I, O2, E>,
    I: Clone,
{
    move |input: I| match first.parse(input.clone()) {
        Ok((input, _)) => {
            let (input, val) = second.parse(input)?;
            return Ok((input, Some(val)));
        }
        Err(_) => Ok((input, None)),
    }
}

/// Runs the first parser and returns it's result in `Either::Left` if condition is true,
/// or runs the second and returns it's with result in `Either::Right` if condition is false.
pub fn either<I, O1, O2, E: ParseError<I>, F, G>(
    b: bool,
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, Either<O1, O2>, E>
where
    F: nom::Parser<I, O1, E>,
    G: nom::Parser<I, O2, E>,
{
    move |input: I| {
        if b {
            let (input, val) = first.parse(input)?;
            Ok((input, Left(val)))
        } else {
            let (input, val) = second.parse(input)?;
            Ok((input, Right(val)))
        }
    }
}
