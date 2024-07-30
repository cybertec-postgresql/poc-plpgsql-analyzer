// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements a Lexer based on the [`logos`] crate.

use std::ops;

use logos::Logos;
use rowan::{TextRange, TextSize};

pub use generated::TokenKind;

mod generated;

/// Wrapper for the actual [`Logos`] parser.
#[derive(Debug)]
pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
}

impl<'a> Lexer<'a> {
    /// Creates a new parsing from an input.
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(input),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?;
        let kind = kind.unwrap();
        let text = self.inner.slice();

        let range = {
            let ops::Range { start, end } = self.inner.span();
            let start = TextSize::try_from(start).unwrap();
            let end = TextSize::try_from(end).unwrap();

            TextRange::new(start, end)
        };

        Some(Self::Item { kind, text, range })
    }
}

/// Represents a single token in the token tree.
#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub range: TextRange,
}

#[cfg(test)]
mod tests {
    use crate::lexer::generated::T;
    use crate::lexer::Lexer;

    use super::*;

    fn check(input: &str, kind: TokenKind) {
        let mut lexer = Lexer::new(input);
        let token = lexer.next().unwrap();
        assert_eq!(token.kind, kind);
        assert_eq!(token.text, input);
    }

    #[test]
    fn lex_spaces_tabs_and_newlines() {
        check(" \t \n", T![whitespace]);
    }

    #[test]
    fn lex_ident() {
        check("hello1$#", T![unquoted_ident]);
    }

    #[test]
    fn lex_quoted_ident() {
        check(r#""读文👩🏼‍🔬""#, T![quoted_ident]);
    }
}
