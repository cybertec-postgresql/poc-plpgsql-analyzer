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

    /// Builds the green node tree, called once parsing is complete
    pub fn build(mut self) -> Parse {
        self.finish();
        Parse {
            green_node: self.builder.finish(),
            _errors: Vec::new(),
        }
    }

    /// Returns the current [`TokenKind`] if there is a token.
    pub(crate) fn peek(&self) -> Option<TokenKind> {
        self.tokens.last().map(|token| token.kind)
    }

    /// Consumes the current token as it is
    pub(crate) fn consume(&mut self) -> Token<'a> {
        assert!(!self.tokens.is_empty());
        let token = self.tokens.pop().unwrap();
        let syntax_kind: SyntaxKind = token.kind.into();
        self.builder.token(syntax_kind.into(), token.text);
        token
    }

    /// Consumes the token with the given [`SyntaxKind`].
    pub(crate) fn consume_as(&mut self, kind: SyntaxKind) -> Token<'a> {
        assert!(!self.tokens.is_empty());
        let token = self.tokens.pop().unwrap();
        self.builder.token(kind.into(), token.text);
        token
    }

    /// Expect the following token, ignore all white spaces inbetween.
    pub(crate) fn expect(&mut self, token_kind: TokenKind) {
        assert!(!self.tokens.is_empty());
        if self.peek().unwrap() == token_kind {
            self.consume();
        } else {
            self.error(token_kind);
        }
    }

    /// Consume all whitespaces / comments & attach
    /// them to the current node to preserve them.
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
        self.start(SyntaxKind::Error);
        let message = if let Some(token_kind) = self.peek() {
            self.consume();
            format!("Expected '{:?}' token, found '{:?}'", expected, token_kind)
        } else {
            format!("Expected '{:?}' token, eof found", expected)
        };
        self.errors.push(message.clone());
        self.finish();
    }
}

/// Implements the [`nom`] internals for implementing the parser.
mod detail {
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
