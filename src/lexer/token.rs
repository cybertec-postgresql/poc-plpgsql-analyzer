// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Token definition for the [`logos`] parser.

use std::fmt;

/// Use to tokenize the input text
#[derive(logos::Logos, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    #[regex("[ \n\r]+")]
    Whitespace,

    #[token("create", ignore(case))]
    CreateKw,

    #[token("procedure", ignore(case))]
    ProcedureKw,

    #[token("or replace", ignore(case))]
    OrReplaceKw,

    #[token("begin", ignore(case))]
    BeginKw,

    #[token("is", ignore(case))]
    IsKw,

    #[token("end", ignore(case))]
    EndKw,

    #[token("in", ignore(case))]
    InKw,

    #[token("out", ignore(case))]
    OutKw,

    #[regex("-?[0-9]+", priority = 2)]
    Integer,

    #[regex("[A-Za-z0-9_][A-Za-z0-9_$.]*")]
    Ident,

    // TODO: Escaped characters, esp. \'
    #[regex("'[^']*'")]
    QuotedLiteral,

    #[token(".")]
    Dot,

    #[token(",")]
    Comma,

    #[token(";")]
    SemiColon,

    #[token(":=")]
    Assign,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("%")]
    Percentage,

    #[token("/")]
    Slash,

    #[regex("--.*")]
    Comment,

    #[error]
    Error,

    /// Marker token to indicate end of input, not used by lexer directly.
    Eof,
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn check(input: &str, kind: TokenKind) {
        let mut lexer = Lexer::new(input);
        let token = lexer.next().unwrap();
        assert_eq!(token.kind, kind);
        assert_eq!(token.text, input);
    }

    #[test]
    fn lex_spaces_and_newlines() {
        check("  \n", TokenKind::Whitespace);
    }

    #[test]
    fn lex_ident() {
        check("hello", TokenKind::Ident);
    }
}
