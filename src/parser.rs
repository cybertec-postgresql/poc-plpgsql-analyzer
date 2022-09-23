// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsers for different SQL language constructs.

use crate::AnalyzeError;
use nom::Finish;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    line: usize,
    column: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProcedureArg {
    span: Span,
    name: String,
    typ: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    ProcedureDef {
        span: Span,
        name: String,
        replace: bool,
        arguments: Vec<ProcedureArg>,
        body: String, // Should eventually be something like `Vec<ParseNode>`
    },
}

#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("Incomplete input; unparsed: {0}")]
    Incomplete(String),
    #[error("Unhandled error: {0}; unparsed: {1}")]
    Unhandled(String, String),
}

impl Span {
    #[cfg(test)]
    fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl<I: ToString> From<nom::error::Error<I>> for ParseError {
    fn from(error: nom::error::Error<I>) -> Self {
        use nom::error::Error;
        use nom::error::ErrorKind;

        match error {
            Error { code: ErrorKind::Eof, input } => Self::Incomplete(input.to_string()),
            Error { code, input } => Self::Unhandled(format!("{:?}", code), input.to_string()),
        }
    }
}

impl From<ParseError> for AnalyzeError {
    fn from(error: ParseError) -> Self {
        AnalyzeError::ParseError(error.to_string())
    }
}

/// Implements the [`nom`] internals for implementing the parser.
mod detail {
    use super::*;
    use nom::branch::alt;
    use nom::bytes::complete::tag_no_case;
    use nom::character::complete::{anychar, char, multispace0, one_of, satisfy};
    use nom::combinator::{all_consuming, map, opt, recognize};
    use nom::multi::{many0, many_till, separated_list0};
    use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
    use nom::{AsChar, IResult, InputTakeAtPosition};

    type LocatedSpan<'a> = nom_locate::LocatedSpan<&'a str>;

    impl From<LocatedSpan<'_>> for Span {
        fn from(span: LocatedSpan<'_>) -> Self {
            Self {
                line: span.location_line() as usize,
                column: span.naive_get_utf8_column(),
            }
        }
    }

    /// A combinator that takes a parser `inner` and produces a parser that also
    /// consumes both leading and trailing whitespace, returning the output
    /// of `inner`.
    fn ws<F, I, O, E>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
    where
        F: Fn(I) -> IResult<I, O, E>,
        I: InputTakeAtPosition,
        <I as InputTakeAtPosition>::Item: AsChar + Clone,
        E: nom::error::ParseError<I>,
    {
        delimited(multispace0, inner, multispace0)
    }

    /// Parses a identifier according to what PostgreSQL calls valid.
    ///
    /// "SQL identifiers and key words must begin with a letter (a-z, but also
    /// letters with diacritical marks and non-Latin letters) or an underscore
    /// (_). Subsequent characters in an identifier or key word can be
    /// letters, underscores, digits (0-9), or dollar signs ($)."
    ///
    /// TODO: Escaped/quoted identifiers
    fn ident(input: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
        let inner = |input| {
            recognize(pair(
                alt((satisfy(|c| c.is_alphabetic()), one_of("0123456789_"))),
                many0(alt((
                    satisfy(|c| c.is_alphabetic()),
                    one_of("0123456789_$"),
                ))),
            ))(input)
        };

        alt((recognize(separated_pair(inner, char('.'), inner)), inner))(input)
    }

    fn procedure_start(input: LocatedSpan) -> IResult<LocatedSpan, (Span, bool, LocatedSpan)> {
        map(
            tuple((
                ws(tag_no_case("create")),
                opt(pair(ws(tag_no_case("or")), ws(tag_no_case("replace")))),
                ws(tag_no_case("procedure")),
                ws(ident),
            )),
            |(_, or_replace, _, name)| (input.into(), or_replace.is_some(), name),
        )(input)
    }

    fn procedure_arg_type(input: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
        alt((recognize(pair(ident, tag_no_case("%type"))), ident))(input)
    }

    fn procedure_arg(input: LocatedSpan) -> IResult<LocatedSpan, ProcedureArg> {
        map(pair(ws(ident), ws(procedure_arg_type)), |(name, typ)| {
            ProcedureArg {
                span: name.into(),
                name: (*name.fragment()).to_owned(),
                typ: (*typ.fragment()).to_owned(),
            }
        })(input)
    }

    fn procedure_args(input: LocatedSpan) -> IResult<LocatedSpan, Vec<ProcedureArg>> {
        delimited(
            char('('),
            separated_list0(char(','), procedure_arg),
            char(')'),
        )(input)
    }

    fn procedure_body(input: LocatedSpan) -> IResult<LocatedSpan, String> {
        preceded(
            tuple((ws(tag_no_case("is")), ws(tag_no_case("begin")))),
            map(
                many_till(
                    recognize(ws(anychar)),
                    tuple((ws(tag_no_case("end")), ws(ident), ws(char(';')))),
                ),
                |(body, _)| {
                    body.into_iter()
                        .map(|ls| *ls.fragment())
                        .collect::<String>()
                },
            ),
        )(input)
    }

    pub fn procedure(input: LocatedSpan) -> IResult<LocatedSpan, Node> {
        all_consuming(map(
            tuple((procedure_start, procedure_args, procedure_body)),
            |((span, replace, name), arguments, body)| Node::ProcedureDef {
                span,
                name: (*name.fragment()).to_owned(),
                replace,
                arguments,
                body,
            },
        ))(input)
    }
}

pub fn parse_procedure(input: &str) -> Result<Node, ParseError> {
    detail::procedure(input.into()).finish()
        .map(|(_, node)| node)
        .map_err(|err| err.into())
}
