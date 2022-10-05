// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsers for different SQL language constructs.

use nom::Finish;
use rowan::GreenNode;

use crate::SyntaxElement;

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

#[derive(Debug, Eq, PartialEq)]
pub struct ProcedureDef {
    pub span: Span,
    pub name: String,
    pub replace: bool,
    pub parameters: Vec<ProcedureParam>,
    pub body: String, // Should eventually be something like `Vec<Node>`
}

/// Represents a single node in the AST.
#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    ProcedureDef(GreenNode),
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

/// Implements the [`nom`] internals for implementing the parser.
mod detail {
    use crate::ast::{leaf, node};
    use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

    use super::*;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, tag_no_case};
    use nom::character::complete::{
        alphanumeric1, anychar, char, line_ending, multispace1, one_of, satisfy,
    };
    use nom::combinator::{all_consuming, map, opt, recognize};
    use nom::multi::{many0, many_till, separated_list0};
    use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};

    /// Custom span as used by parser internals.
    type IResult<'a> = nom::IResult<&'a str, SyntaxElement>;

    /// Parses white space characters
    fn ws(input: &str) -> IResult {
        map(multispace1, |ws| leaf(SyntaxKind::Whitespace, ws))(input)
    }

    /// Parses an inline comment
    fn comment(input: &str) -> IResult {
        // TODO fix to return full comment
        map(
            tuple((tag("-"), tag("-"), many_till(anychar, line_ending))),
            |(_, _, s)| {
                println!("COMMENT: {:?}", s);
                leaf(SyntaxKind::Comment, s.1)
            },
        )(input)
    }

    /// Parses the left paren
    fn lparen(input: &str) -> IResult {
        map(tag("("), |s| leaf(SyntaxKind::LeftParen, s))(input)
    }

    /// Parses the right paren
    fn rparen(input: &str) -> IResult {
        map(tag(")"), |s| leaf(SyntaxKind::RightParen, s))(input)
    }

    /// Parses a identifier according to what PostgreSQL calls valid.
    ///
    /// "SQL identifiers and key words must begin with a letter (a-z, but also
    /// letters with diacritical marks and non-Latin letters) or an underscore
    /// (_). Subsequent characters in an identifier or key word can be
    /// letters, underscores, digits (0-9), or dollar signs ($)."
    ///
    /// TODO: Escaped/quoted identifiers
    fn ident(input: &str) -> IResult {
        let inner = |input| {
            recognize(pair(
                alt((satisfy(|c| c.is_alphabetic()), one_of("0123456789_"))),
                many0(alt((
                    satisfy(|c| c.is_alphabetic()),
                    one_of("0123456789_$"),
                ))),
            ))(input)
        };

        map(
            alt((recognize(separated_pair(inner, char('.'), inner)), inner)),
            |s| leaf(SyntaxKind::Ident, s),
        )(input)
    }

    /// Parses the start of a procedure, including the procedure name.
    fn procedure_start(input: &str) -> IResult {
        map(
            tuple((
                opt(comment),
                opt(ws),
                tag_no_case("create"),
                ws,
                opt(pair(tag_no_case("or replace"), ws)),
                tag_no_case("procedure"),
                ws,
                ident,
            )),
            |(c1, ws1, kw_create, ws2, replace, kw_procedure, ws3, fn_name)| {
                let mut children: Vec<SyntaxElement> = Vec::new();
                if let Some(comment) = c1 {
                    children.push(comment);
                }
                if let Some(ws) = ws1 {
                    children.push(ws);
                }
                children.push(leaf(SyntaxKind::Keyword, kw_create));
                children.push(ws2);
                if let Some((replace, ws)) = replace {
                    children.push(leaf(SyntaxKind::Keyword, replace));
                    children.push(ws);
                }
                children.push(leaf(SyntaxKind::Keyword, kw_procedure));
                children.push(ws3);
                children.push(fn_name);
                node(SyntaxKind::ProcedureStart, children)
            },
        )(input)
    }

    /// Parses a single procedure parameter type, either a base type or a column
    /// reference.
    /*
    fn procedure_param_type(input: LocatedSpan) -> IResult<LocatedSpan, LocatedSpan> {
        alt((recognize(pair(ident, tag_no_case("%type"))), ident))(input)
    }
    */

    /// Parses a single procedure paramter, i.e. name and it's datatype.
    /*
    fn procedure_param(input: LocatedSpan) -> IResult<LocatedSpan, ProcedureParam> {
        map(pair(ws(ident), ws(procedure_param_type)), |(name, typ)| {
            ProcedureParam {
                span: name.into(),
                name: (*name.fragment()).to_owned(),
                typ: (*typ.fragment()).to_owned(),
            }
        })(input)
    }
    */

    /// Parses a list of procedure parameters, as surrounded by `(` and `)`.
    /*
    fn procedure_params(input: LocatedSpan) -> IResult<LocatedSpan, Vec<ProcedureParam>> {
        map(
            opt(delimited(
                char('('),
                separated_list0(char(','), procedure_param),
                char(')'),
            )),
            Option::unwrap_or_default,
        )(input)
    }
    */

    /// Parses the body of a procedure, that is anything between `IS BEGIN` and
    /// `END <name>;`.
    /*
    fn procedure_body<'a, E>(
        input: LocatedSpan<'a>,
        name: &str,
    ) -> IResult<LocatedSpan<'a>, String, E>
    where
        E: nom::error::ParseError<LocatedSpan<'a>>,
    {
        all_consuming(preceded(
            tuple((ws(tag_no_case("is")), ws(tag_no_case("begin")))),
            map(
                many_till(
                    recognize(anychar::<LocatedSpan<'a>, E>),
                    tuple((tag_no_case("end"), ws(tag_no_case(name)), ws(char(';')))),
                ),
                |(body, _)| {
                    body.into_iter()
                        .map(|ls| *ls.fragment())
                        .collect::<String>()
                },
            ),
        ))(input)
    }
    */

    /// Parses an complete PL/SQL procedure.
    pub fn procedure(input: &str) -> IResult {
        let mut children = Vec::new();
        let (input, result) = procedure_start(input)?;

        children.push(result);

        Ok((input, node(SyntaxKind::Root, children)))
        /*
        let (input, ((span, replace, name), parameters)) =
            pair(procedure_start, procedure_params)(input)?;
        let (input, body) = procedure_body(input, name.fragment())?;

        Ok((
            input,
            Node::ProcedureDef(ProcedureDef {
                span,
                name: (*name.fragment()).to_owned(),
                replace,
                parameters,
                body,
            }),
        ))
        */
    }

    #[cfg(test)]
    mod tests {
        use crate::parser::detail::{comment, ident, ws};

        use super::procedure_start;

        #[test]
        fn parses_whitespace() {
            assert!(ws("   ").is_ok());
            assert!(ws("a").is_err());
        }

        #[test]
        fn parses_inline_comments() {
            assert!(comment("-- hello\n").is_ok());
            assert!(comment("hello\n").is_err());
        }

        #[test]
        fn parse_identifiers() {
            assert!(ident("abc").is_ok());
            assert!(ident("abc1").is_ok());
            assert!(ident("_abc2").is_ok());
            assert!(ident("abc.123").is_ok());
        }

        #[test]
        fn parse_procedure_start() {
            assert!(procedure_start("CREATE PROCEDURE hello").is_ok());
            assert!(procedure_start(" CREATE PROCEDURE bar").is_ok());
            assert!(procedure_start(" CREATE OR REPLACE PROCEDURE foo").is_ok());
        }

        #[test]
        fn parse_procedure_start_with_comment() {
            const INPUT: &str = "-- This is a test\nCREATE PROCEDURE hello";
            let result = procedure_start(INPUT);
            assert!(result.is_ok(), "{:#?}", result);
            let (_text, node) = result.unwrap();
            println!("{}", node.to_string());
        }

        #[test]
        fn reject_invalid_procedure_start() {
            assert!(procedure_start("PROCEDURE CREATE hello").is_err());
        }
    }
}

/// Public entry point for parsing a complete PL/SQL procedure.
pub fn parse_procedure(input: &str) -> Result<SyntaxElement, ParseError> {
    detail::procedure(input.into())
        .finish()
        .map(|(_, node)| node)
        .map_err(Into::into)
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
        dbg!(&result);
        /*
        assert_eq!(
            result.unwrap(),
            Node::ProcedureDef(ProcedureDef {
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
            }),
        );
        */
    }
}
