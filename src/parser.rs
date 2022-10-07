// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsers for different SQL language constructs.

use rowan::{GreenNode, GreenNodeBuilder};
use crate::{SyntaxKind, ast::SyntaxNode, Token, Lexer, lexer::TokenKind, grammar::parse_procedure};

/// Represents a single node in the AST.
#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    /// TODO replace with a cast-able Procedure that maps the syntax node
    ProcedureDef(SyntaxNode),
}

/// Error type describing all possible parser failures.
#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum ParseError {
    /// The input is incomplete, i.e. it could not be fully parsed through.
    #[error("Incomplete input; unparsed: {0}")]
    Incomplete(String),
    /// A token could not be parsed by the lexer
    #[error("Token is not known: {0}")]
    UnknownToken(String),
    /// The parser expected a specifc token, but found another.
    #[error("Expected token '{0}'")]
    ExpectedToken(TokenKind),
    /// The parser stumbled upon the end of input, but expecting further input.
    #[error("Unexpected end of input found")]
    Eof,
    /// Any parser error currently not described further ("catch-all").
    #[error("Unhandled error: {0}; unparsed: {1}")]
    Unhandled(String, String),
}

/// Main function to parse the input string.
pub fn parse(input: &str) -> Result<Parse, ParseError> {
    let mut tokens = Lexer::new(input).collect::<Vec<_>>();
    tokens.reverse();
    let mut parser = Parser::new(tokens);

    // Expect a procedure
    parse_procedure(&mut parser);

    // TODO handle any errors here
    Ok(parser.build())
}

/// The struct helds the parsed / built green syntax tree with
/// a list of parse errors.
#[derive(Debug)]
pub struct Parse {
    green_node: GreenNode,
    pub errors: Vec<ParseError>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }

    #[allow(unused)]
    pub fn tree(&self) -> String {
        format!("{:#?}", self.syntax())
    }

    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub struct Parser<'a> {
    /// All tokens generated from a Lexer.
    tokens: Vec<Token<'a>>,
    /// The in-progress tree builder
    builder: GreenNodeBuilder<'static>,
    /// The list of all found errors
    errors: Vec<ParseError>,
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

    /// Builds the green node tree, called once the parsing is complete
    pub fn build(mut self) -> Parse {
        if !self.tokens.is_empty() {
            let remaining_tokens = self.tokens.iter().map(|t| t.text).collect::<String>();
            let error = ParseError::Incomplete(remaining_tokens);
            self.start(SyntaxKind::Error);
            self.errors.push(error);
            self.finish();
        }

        self.finish();
        Parse {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    /// Checks if the current token is `kind`.
    pub fn at(&mut self, kind: TokenKind) -> bool {
        self.eat_ws();
        if let Some(token) = self.tokens.last() {
            token.kind == kind
        } else {
            false
        }
    }

    /// Returns the current [`TokenKind`] if there is a token.
    pub fn current(&mut self) -> Option<TokenKind> {
        self.eat_ws();
        self.tokens.last().map(|token| token.kind)
    }

    /// Consumes the current token as it is
    pub fn bump(&mut self) -> Token<'a> {
        let token = self.tokens.pop().unwrap();
        let syntax_kind: SyntaxKind = token.kind.into();
        self.builder.token(syntax_kind.into(), token.text);
        token
    }

    /// Consumes all tokens until the last searched token is found.
    pub fn until_last(&mut self, token_kind: TokenKind) {
        // The tokens list is reversed, therefore the search is done from front.
        if let Some(index) = self.tokens.iter().position(|token| token.kind == token_kind) {
            while self.tokens.len() > (index + 1) {
                self.bump();
            }
        } else {
            self.error(ParseError::ExpectedToken(token_kind));
        }
    }

    /// Expect the following token, ignore all white spaces inbetween.
    pub fn expect(&mut self, token_kind: TokenKind) {
        match self.current() {
            Some(kind) if kind == token_kind => {
                self.bump();
            },
            _ => self.error(ParseError::ExpectedToken(token_kind)),
        }
    }

    /// Consume all whitespaces / comments & attach
    /// them to the current node to preserve them.
    pub fn eat_ws(&mut self) {
        loop {
            match self.tokens.last() {
                Some(token) if token.kind.is_trivia() => {
                    let token = self.tokens.pop().unwrap();
                    let syntax_kind: SyntaxKind = token.kind.into();
                    self.builder.token(syntax_kind.into(), token.text);
                },
                _ => break,
            }
        }
    }

    pub(crate) fn start(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    pub(crate) fn finish(&mut self) {
        self.builder.finish_node();
        self.eat_ws();
    }

    /// Mark the given error.
    fn error(&mut self, error: ParseError) {
        self.start(SyntaxKind::Error);
        self.builder.token(SyntaxKind::Text.into(), error.to_string().as_str());
        self.errors.push(error);
        self.finish();
    }
}
