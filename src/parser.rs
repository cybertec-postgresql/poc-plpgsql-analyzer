// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsers for different SQL language constructs.

use rowan::{GreenNode, GreenNodeBuilder};
use crate::{SyntaxKind, ast::SyntaxNode, Token, Lexer, lexer::TokenKind};

/// Represents a single node in the AST.
#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    ProcedureDef(SyntaxNode),
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

/// Main function to parse the input string.
pub fn parse(input: &str) -> Result<Parse, ParseError> {
    let tokens = Lexer::new(input).collect::<Vec<_>>();
    let parser = Parser::new(tokens);

    // TODO handle any errors here
    Ok(parser.build())
}

#[derive(Debug)]
pub struct Parse {
    green_node: GreenNode,
    _errors: Vec<ParseError>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }

    pub fn tree(&self) -> String {
        format!("{:#?}", self.syntax())
    }
}

pub(crate) struct Parser<'a> {
    /// The lexer to get tokens
    tokens: Vec<Token<'a>>,
    /// The in-progress tree builder
    builder: GreenNodeBuilder<'static>,
    /// The list of all found errors
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        let mut parser = Parser {
            tokens,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        };
        parser.start(SyntaxKind::Root);
        parser
    }

    /// Builds the green node tree
    pub fn build(mut self) -> Parse {
        self.finish();
        Parse {
            green_node: self.builder.finish(),
            _errors: Vec::new(),
        }
    }

    pub(crate) fn peek(&self) -> Option<TokenKind> {
        self.tokens.last().map(|token| token.kind)
    }

    /// Consumes the current token
    pub(crate) fn consume(&mut self) {
        assert!(!self.tokens.is_empty());
        let token = self.tokens.pop().unwrap();
        let syntax_kind: SyntaxKind = token.kind.into();
        self.builder.token(syntax_kind.into(), token.text);
    }

    /// Consume all whitespaces / comments
    pub(crate) fn eat_ws(&mut self) {
        loop {
            match self.peek() {
                Some(token) if token.is_trivia() => {
                    self.consume();
                },
                _ => break,
            }
        }
    }

    pub(crate) fn start(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    pub(crate) fn finish(&mut self) {
        self.builder.finish_node()
    }

    /// Mark the current token as error
    pub(crate) fn error(&mut self, expected: TokenKind) {
        let message = format!("Found '{:?}' token, expected was '{:?}'", self.peek().unwrap(), expected);
        self.consume();
        self.errors.push(message);
    }
}

/// Implements the [`nom`] internals for implementing the parser.
mod detail {
    /*
    /// Parses a single comma
    fn comma(input: &str) -> IResult {
        map(tag(","), |s| leaf(SyntaxKind::Comma, s))(input)
    }

    /// Parses the left paren
    fn lparen(input: &str) -> IResult {
        map(tag("("), |s| leaf(SyntaxKind::LeftParen, s))(input)
    }

    /// Parses the right paren
    fn rparen(input: &str) -> IResult {
        map(tag(")"), |s| leaf(SyntaxKind::RightParen, s))(input)
    }
    */

    /*
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
    */

    /*
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
    */

    /*
    /// Parses a single procedure parameter type, either a base type or a column
    /// reference.
    fn procedure_param_type(input: &str) -> IResult {
        map(
            tuple((
                opt(ws),
                opt(comment),
                opt(ws),
                alt((
                    map(recognize(pair(ident, tag_no_case("%type"))), |s| {
                        leaf(SyntaxKind::Ident, s)
                    }),
                    ident,
                )),
            )),
            |(ws1, comment, ws2, var_type)| {
                let mut children = Vec::new();
                if let Some(ws) = ws1 {
                    children.push(ws);
                }
                if let Some(comment) = comment {
                    children.push(comment);
                }
                if let Some(ws) = ws2 {
                    children.push(ws);
                }
                children.push(var_type);
                node(SyntaxKind::ParamType, children)
            },
        )(input)
    }
    */

    /*
    /// Parses a single procedure paramter, i.e. name and it's datatype.
    fn procedure_param(input: &str) -> IResult {
        map(
            tuple((opt(ws), opt(comment), opt(ws), ident, procedure_param_type)),
            |(ws1, comment, ws2, param_name, param_type)| {
                let mut children = Vec::new();
                if let Some(ws) = ws1 {
                    children.push(ws);
                }
                if let Some(comment) = comment {
                    children.push(comment);
                }
                if let Some(ws) = ws2 {
                    children.push(ws);
                }
                children.push(param_name);
                children.push(param_type);
                node(SyntaxKind::Param, children)
            },
        )(input)
    }
    */

    /*
    /// Parses a list of procedure parameters, as surrounded by `(` and `)`.
    fn procedure_param_list(input: &str) -> IResult {
        map(
            opt(delimited(
                lparen,
                separated_list0(comma, procedure_param),
                rparen,
            )),
            |s| {
                let mut children = Vec::new();
                if let Some(mut nodes) = s {
                    children.append(&mut nodes);
                }
                node(SyntaxKind::ParamList, children)
            },
        )(input)
    }
    */

    /*
    /// Parses the procedure header
    fn procedure_header(input: &str) -> IResult {
        map(
            tuple((opt(ws), procedure_start, opt(ws), procedure_param_list)),
            |(ws1, start, ws2, params)| {
                let mut children = Vec::new();
                if let Some(ws) = ws1 {
                    children.push(ws);
                }
                children.push(start);
                if let Some(ws) = ws2 {
                    children.push(ws);
                }
                children.push(params);
                node(SyntaxKind::ProcedureHeader, children)
            },
        )(input)
    }
    */

    /*
    /// Parses the body of a procedure, that is anything between `IS BEGIN` and
    /// `END <name>;`.
    ///
    /// For example:
    /// `let result = procedure_body("IS BEGIN\nEND hello;", "hello");`
    ///
    fn procedure_body<'a>(input: &'a str, fn_name: &str) -> IResult<'a> {
        println!("PROCEDURE_BODY: {}\nfn_name: {}", input, fn_name);
        map(
            tuple((
                opt(ws),
                tag_no_case("is"),
                ws,
                tag_no_case("begin"),
                map(
                    many_till(
                        recognize(anychar),
                        tuple((tag_no_case("end"), ws, tag(fn_name), opt(ws), char(';'))),
                    ),
                    |(body, (kw_end, ws1, fn_name, ws2, colon))| {
                        let body = body.into_iter().map(String::from).collect::<String>();
                        let mut children = Vec::new();
                        children.push(leaf(SyntaxKind::Text, body.as_str()));
                        children.push(leaf(SyntaxKind::Keyword, kw_end));
                        children.push(ws1);
                        children.push(leaf(SyntaxKind::Ident, fn_name));
                        if let Some(ws) = ws2 {
                            children.push(ws);
                        }
                        children.push(leaf(SyntaxKind::SemiColon, colon.to_string().as_str()));
                        node(SyntaxKind::Unsupported, children)
                    },
                ),
            )),
            |(ws1, kw_is, ws2, kw_begin, body)| {
                let mut children = Vec::new();
                if let Some(ws) = ws1 {
                    children.push(ws);
                }
                children.push(leaf(SyntaxKind::Keyword, kw_is));
                children.push(ws2);
                children.push(leaf(SyntaxKind::Keyword, kw_begin));
                children.push(body);
                node(SyntaxKind::ProcedureBody, children)
            },
        )(input)
    }
    */

    /*
    /// Parses an complete PL/SQL procedure.
    pub fn procedure(input: &str) -> IResult {
        let mut children = Vec::new();
        // let (input, header) = procedure_header(input)?;
        children.push(header);

        // TODO get the procedure name, pass into body

        // let (input, result) = procedure_body(input, "hello")?;
        children.push(result);
        Ok((input, node(SyntaxKind::Root, children)))
    }
    */

    /*
    #[cfg(test)]
    mod tests {
        use crate::{
            parser::detail::{
                ident, procedure_body, procedure_header, procedure_param, procedure_param_list,
            },
            SyntaxKind,
        };

        use super::{procedure_param_type, procedure_start};

        #[test]
        fn parse_whitespace() {
            assert!(ws("   ").is_ok());
            assert!(ws("a").is_err());
        }

        #[test]
        fn parse_inline_comments() {
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
        }

        #[test]
        fn reject_invalid_procedure_start() {
            assert!(procedure_start("PROCEDURE CREATE hello").is_err());
        }

        #[test]
        fn parse_valid_procedure_param_type() {
            assert!(procedure_param_type("hello").is_ok());
            assert!(procedure_param_type("   lisa12").is_ok());
            assert!(procedure_param_type("hello%type").is_ok());
            assert!(procedure_param_type("-- test\n   hello%type").is_ok());
        }

        #[test]
        fn parse_single_procedure_param() {
            assert!(procedure_param("p_1 VARCHAR2").is_ok());
            assert!(procedure_param("  foo -- comment\n  bar%type").is_ok());
        }

        #[test]
        fn parse_procedure_param_list() {
            assert!(procedure_param_list("()").is_ok());
            assert!(procedure_param_list("( hello my%type )").is_ok());

            const INPUT: &str = "( first var%type , second other_type )";
            let (_, node) = procedure_param_list(INPUT).unwrap();
            assert_eq!(node.kind(), SyntaxKind::ParamList.into());
        }

        #[test]
        fn parse_procedure_body() {
            let result = procedure_body("IS BEGIN\nEND hello;", "hello");
            dbg!(&result);

            assert!(procedure_body("IS BEGIN\nEND hello;", "hello").is_ok());
            assert!(procedure_body("IS BEGIN\nEND foo;", "bar").is_err());
            assert!(procedure_body("IS BEGIN\nNULL\nEND hello;", "hello").is_ok());
        }

        #[test]
        fn parse_procedure_header() {
            const INPUT: &str = r#"
            CREATE OR REPLACE PROCEDURE add_job_history
              (  p_emp_id          job_history.employee_id%type
               , p_start_date      job_history.start_date%type
               )"#;
            let result = procedure_header(INPUT);
            dbg!(&result);
            assert!(result.is_ok());
        }
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;
    // use pretty_assertions::assert_eq;

    const ADD_JOB_HISTORY: &str = include_str!("../tests/fixtures/add_job_history.sql");
    const ADD_JOB_HISTORY_BODY: &str = include_str!("../tests/fixtures/add_job_history_body.sql");

    #[test]
    fn test_parse_procedure() {
        let result = parse(ADD_JOB_HISTORY);
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
