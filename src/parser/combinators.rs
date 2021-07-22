use alloc::vec::Vec;

use bitvec::prelude::*;
use either::*;
use nom::combinator::cond;
use nom::error::ParseError;
use nom::multi::count;
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

/// Runs the given parser for each `true` in the given `BitVec` and pushes a `Some(parser_retval)`.
/// For each `false`, does not run the parser and pushes a `None`.
pub fn many_cond_opt<I, O, E, F>(f: F, bv: BitVec) -> impl FnMut(I) -> IResult<I, Vec<Option<O>>, E>
where
    F: nom::Parser<I, O, E> + Clone,
    I: Clone + PartialEq,
    O: Sized,
    E: ParseError<I>,
{
    move |input: I| {
        let mut iter = bv.iter();
        let (input, ret): (I, Vec<Option<O>>) =
            count(cond(*iter.next().unwrap(), f.clone()), bv.len())(input)?;
        return Ok((input, ret));
    }
}
