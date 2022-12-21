// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements parsers for different SQL language constructs.

use rowan::{Checkpoint, GreenNode, GreenNodeBuilder};

use crate::grammar;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::syntax::{SyntaxKind, SyntaxNode};

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
    /// The parser expected one of many tokens, but found neither of them.
    #[error("Expected one of tokens: '{0:?}")]
    ExpectedOneOfTokens(Vec<TokenKind>),
    /// The parser stumbled upon an unbalanced pair of parentheses.
    #[error("Unbalanced pair of parentheses found")]
    UnbalancedParens,
    /// The parser stumbled upon the end of input, but expecting further input.
    #[error("Unexpected end of input found")]
    Eof,
    /// Any parser error currently not described further ("catch-all").
    #[error("Unhandled error: {0}; unparsed: {1}")]
    Unhandled(String, String),
}

/// Tries to parse any string of SQL tokens.
pub fn parse_any(input: &str) -> Result<Parse, ParseError> {
    let mut parser = Parser::new(input);

    while !parser.at(TokenKind::Eof) {
        parser.bump_any();
    }

    // TODO handle any errors here
    Ok(parser.build())
}

/// Tries to parse a function from a string.
pub fn parse_function(input: &str) -> Result<Parse, ParseError> {
    let mut parser = Parser::new(input);

    // Expect a function
    grammar::parse_function(&mut parser);

    // TODO handle any errors here
    Ok(parser.build())
}

/// Tries to parse a procedure from a string.
pub fn parse_procedure(input: &str) -> Result<Parse, ParseError> {
    let mut parser = Parser::new(input);

    // Expect a procedure
    grammar::parse_procedure(&mut parser);

    // TODO handle any errors here
    Ok(parser.build())
}

pub fn parse_query(input: &str) -> Result<Parse, ParseError> {
    let mut parser = Parser::new(input);

    // Expect a query `SELECT`
    grammar::parse_query(&mut parser);

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
    pub fn new(input: &'a str) -> Self {
        let tokens = Lexer::new(input).collect::<Vec<_>>();
        Self::from_tokens(tokens)
    }

    pub fn from_tokens(mut tokens: Vec<Token<'a>>) -> Self {
        tokens.reverse();
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

    /// Lookahead operation: returns the kind of the next nth token.
    pub fn nth(&mut self, mut n: usize) -> Option<TokenKind> {
        let mut i = 1;
        loop {
            match &self.tokens.iter().rev().peekable().nth(i) {
                Some(token) => {
                    if !token.kind.is_trivia() {
                        n -= 1;
                        if n == 0 {
                            return Some(token.kind);
                        }
                    }
                    i += 1;
                }
                None => {
                    return None;
                }
            }
        }
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

    /// Consumes the next token if `kind` matches.
    pub fn eat_one_of(&mut self, kinds: &[TokenKind]) -> bool {
        if !kinds.contains(&self.current()) {
            return false;
        }
        self.do_bump();
        true
    }

    /// Consumes the next token if `kind` matches and creates a new node of
    /// `target`
    pub fn eat_one_of_map(&mut self, kinds: &[TokenKind], target: SyntaxKind) -> bool {
        if !kinds.contains(&self.current()) {
            false
        } else {
            self.do_bump_map(target);
            true
        }
    }

    /// Consumes the current token as it is
    pub fn bump(&mut self, kind: TokenKind) {
        assert!(self.eat(kind));
    }

    /// Consumes the next token, advances by one token
    pub fn bump_any(&mut self) {
        if self.current() != TokenKind::Eof {
            self.do_bump();
        }
    }

    /// Consumes the next token as `target`, advances by one token
    pub fn bump_any_map(&mut self, target: SyntaxKind) {
        if self.current() != TokenKind::Eof {
            self.do_bump_map(target);
        }
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

    /// Expect one of the following tokens, ignore all white spaces inbetween.
    pub fn expect_one_of(&mut self, token_kinds: &[TokenKind]) -> bool {
        if token_kinds.contains(&self.current()) {
            self.do_bump();
            return true;
        }

        self.error(ParseError::ExpectedOneOfTokens(token_kinds.to_vec()));
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

    /// Start a new (nested) node at a checkpoint
    pub(crate) fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into())
    }

    pub(crate) fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }

    /// Finish the current node
    pub(crate) fn finish(&mut self) {
        self.builder.finish_node();
        self.eat_ws();
    }

    /// Mark the given error.
    pub(crate) fn error(&mut self, error: ParseError) {
        self.start(SyntaxKind::Error);
        self.builder
            .token(SyntaxKind::Text.into(), error.to_string().as_str());
        self.errors.push(error);
        self.finish();
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

    /// Function to consume the next token, regardless of any [`TokenKind`], and
    /// add it as `target` `[SyntaxKind]` node to the tree
    fn do_bump_map(&mut self, target: SyntaxKind) {
        assert!(!self.tokens.is_empty());
        let token = self.tokens.pop().unwrap();
        if token.kind == TokenKind::Error {
            self.error(ParseError::UnknownToken(token.text.to_string()));
        }
        self.builder.token(target.into(), token.text);
    }
}
