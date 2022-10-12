// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements parsers for different SQL language constructs.

use crate::grammar::parse_procedure;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::syntax::{SyntaxKind, SyntaxNode};
use rowan::{GreenNode, GreenNodeBuilder};

/// Error type describing all possible parser failures.
#[derive(Debug, Eq, thiserror::Error, PartialEq)]
pub enum ParseError {
    /// The input is incomplete, i.e. it could not be fully parsed through.
    #[error("Incomplete input; unparsed: {0}")]
    Incomplete(String),
    /// A token could not be parsed by the lexer
    #[error("Unknown token found: {0}")]
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

/// The struct holds the parsed / built green syntax tree with
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

/// A custom parser to build a green Syntax Tree from a list
/// of tokens.
pub struct Parser<'a> {
    /// All tokens generated from a Lexer.
    tokens: Vec<Token<'a>>,
    /// The in-progress tree builder
    builder: GreenNodeBuilder<'static>,
    /// The list of all found errors.
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
            self.error(ParseError::Incomplete(remaining_tokens));
        }

        self.finish();
        Parse {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    /// Checks if the current token is `kind`.
    pub fn at(&mut self, kind: TokenKind) -> bool {
        self.current() == kind
    }

    /// Returns the current [`TokenKind`] if there is a token.
    pub fn current(&mut self) -> TokenKind {
        self.eat_ws();
        match self.tokens.last() {
            Some(token) => token.kind,
            None => TokenKind::Eof,
        }
    }

    /// Consumes the next token if `kind` matches.
    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        self.do_bump();
        true
    }

    /// Consumes the current token as it is
    pub fn bump(&mut self, kind: TokenKind) {
        assert!(self.eat(kind));
    }

    /// Consumes the next token, advances by one token
    pub fn bump_any(&mut self) {
        if self.current() == TokenKind::Eof {
            return;
        }
        self.do_bump();
    }

    /// Consumes all tokens until the last searched token is found.
    pub fn until_last(&mut self, token_kind: TokenKind) {
        // The tokens list is reversed, therefore the search is done from front.
        if let Some(index) = self
            .tokens
            .iter()
            .position(|token| token.kind == token_kind)
        {
            while self.tokens.len() > (index + 1) {
                self.do_bump();
            }
        } else {
            self.error(ParseError::ExpectedToken(token_kind));
        }
    }

    /// Expect the following token, ignore all white spaces inbetween.
    pub fn expect(&mut self, token_kind: TokenKind) -> bool {
        if self.eat(token_kind) {
            return true;
        }
        self.error(ParseError::ExpectedToken(token_kind));
        false
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
                }
                _ => break,
            }
        }
    }

    /// Start a new (nested) node
    pub(crate) fn start(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    /// Finish the current node
    pub(crate) fn finish(&mut self) {
        self.builder.finish_node();
        self.eat_ws();
    }

    /// Function to consume the next token, regardless of any [`TokenKind`]
    fn do_bump(&mut self) {
        assert!(!self.tokens.is_empty());
        let token = self.tokens.pop().unwrap();
        if token.kind == TokenKind::Error {
            self.error(ParseError::UnknownToken(token.text.to_string()));
        }
        let syntax_kind: SyntaxKind = token.kind.into();
        self.builder.token(syntax_kind.into(), token.text);
    }

    /// Mark the given error.
    fn error(&mut self, error: ParseError) {
        self.start(SyntaxKind::Error);
        self.builder
            .token(SyntaxKind::Text.into(), error.to_string().as_str());
        self.errors.push(error);
        self.finish();
    }
}
