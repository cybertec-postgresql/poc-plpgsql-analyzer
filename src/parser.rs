// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsers for different SQL language constructs.

use nom::Finish;
use std::fmt;

/// A specific location in the input data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    line: usize,
    column: usize,
}

/// An parameter in a procedure definition.
#[derive(Debug, Eq, PartialEq)]
pub struct ProcedureParam {
    span: Span,
    name: String,
    typ: String,
}

/// Represents a single node in the AST.
#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    ProcedureDef {
        span: Span,
        name: String,
        replace: bool,
        parameters: Vec<ProcedureParam>,
        body: String, // Should eventually be something like `Vec<Node>`
    },
}

/// Error type describing all possible parser failures.
#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum ParseError {
    /// The input is incomplete, i.e. it could not be fully parsed through.
    #[error("Incomplete input; unparsed: {0}")]
    Incomplete(String),
    /// Any parser error currently not described further ("catch-all").
    #[error("Unhandled error: {0}; unparsed: {1}")]
    Unhandled(String, String),
}

impl Span {
    /// Currently only used by tests.
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
            Error {
                code: ErrorKind::Eof,
                input,
            } => Self::Incomplete(input.to_string()),
            Error { code, input } => Self::Unhandled(format!("{:?}", code), input.to_string()),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::ProcedureDef { .. } => write!(f, "ProcedureDef"),
        }
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

    /// Custom span as used by parser internals.
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

    /// Parses the start of a procedure, including the procedure name.
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

    /// Parses a single procedure parameter type, either a base type or a column
    /// reference.
    fn procedure_param_type(input: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
        alt((recognize(pair(ident, tag_no_case("%type"))), ident))(input)
    }

    /// Parses a single procedure paramter, i.e. name and it's datatype.
    fn procedure_param(input: LocatedSpan) -> IResult<LocatedSpan, ProcedureParam> {
        map(pair(ws(ident), ws(procedure_param_type)), |(name, typ)| {
            ProcedureParam {
                span: name.into(),
                name: (*name.fragment()).to_owned(),
                typ: (*typ.fragment()).to_owned(),
            }
        })(input)
    }

    /// Parses a list of procedure parameters, as surrounded by `(` and `)`.
    fn procedure_params(input: LocatedSpan) -> IResult<LocatedSpan, Vec<ProcedureParam>> {
        delimited(
            char('('),
            separated_list0(char(','), procedure_param),
            char(')'),
        )(input)
    }

    /// Parses the body of a procedure, that is anything between `IS BEGIN` and
    /// `END <name>;`.
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

    /// Parses an complete PL/SQL procedure.
    pub fn procedure(input: LocatedSpan) -> IResult<LocatedSpan, Node> {
        all_consuming(map(
            tuple((procedure_start, procedure_params, procedure_body)),
            |((span, replace, name), parameters, body)| Node::ProcedureDef {
                span,
                name: (*name.fragment()).to_owned(),
                replace,
                parameters,
                body,
            },
        ))(input)
    }
}

/// Public entry point for parsing a complete PL/SQL procedure.
pub fn parse_procedure(input: &str) -> Result<Node, ParseError> {
    detail::procedure(input.into())
        .finish()
        .map(|(_, node)| node)
        .map_err(|err| err.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const ADD_JOB_HISTORY: &str = include_str!("../tests/fixtures/add_job_history.sql");
    const ADD_JOB_HISTORY_BODY: &str = include_str!("../tests/fixtures/add_job_history_body.sql");

    #[test]
    fn test_parse_procedure() {
        let result = parse_procedure(ADD_JOB_HISTORY);
        assert!(result.is_ok(), "{:#?}", result);
        assert_eq!(
            result.unwrap(),
            Node::ProcedureDef {
                span: Span::new(1, 1),
                name: "add_job_history".into(),
                replace: true,
                parameters: vec![
                    ProcedureParam {
                        span: Span::new(2, 6),
                        name: "p_emp_id".into(),
                        typ: "job_history.employee_id%type".into(),
                    },
                    ProcedureParam {
                        span: Span::new(3, 6),
                        name: "p_start_date".into(),
                        typ: "job_history.start_date%type".into(),
                    },
                    ProcedureParam {
                        span: Span::new(4, 6),
                        name: "p_end_date".into(),
                        typ: "job_history.end_date%type".into(),
                    },
                    ProcedureParam {
                        span: Span::new(5, 6),
                        name: "p_job_id".into(),
                        typ: "job_history.job_id%type".into(),
                    },
                    ProcedureParam {
                        span: Span::new(6, 6),
                        name: "p_department_id".into(),
                        typ: "job_history.department_id%type".into(),
                    },
                ],
                body: ADD_JOB_HISTORY_BODY.into(),
            },
        );
    }
}
