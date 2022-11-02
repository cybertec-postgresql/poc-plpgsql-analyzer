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

    #[token("function", ignore(case))]
    FunctionKw,

    #[token("replace", ignore(case))]
    ReplaceKw,

    #[token("begin", ignore(case))]
    BeginKw,

    #[token("is", ignore(case))]
    IsKw,

    #[token("as", ignore(case))]
    AsKw,

    #[token("$$")]
    DollarQuote,

    #[token("end", ignore(case))]
    EndKw,

    #[token("in", ignore(case))]
    InKw,

    #[token("out", ignore(case))]
    OutKw,

    #[token("return", ignore(case))]
    ReturnKw,

    #[token("deterministic", ignore(case))]
    DeterministicKw,

    #[token("type", ignore(case))]
    TypeKw,

    #[token("number", ignore(case))]
    NumberKw,

    #[token("select", ignore(case))]
    SelectKw,

    #[token("from", ignore(case))]
    FromKw,

    #[token("where", ignore(case))]
    WhereKw,

    #[token("and", ignore(case))]
    AndKw,

    #[token("or", priority = 100, ignore(case))]
    OrKw,

    #[regex(r"(?i)i?like")]
    LikeKw,

    #[token("(+)")]
    OracleJoinKw,

    #[regex("-?[0-9]+")]
    Integer,

    #[regex(r"(?i)[a-z_][a-z0-9_$]*(\.[a-z_][a-z0-9_$]*)?", priority = 1)]
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

    #[token("*")]
    Asterisk,

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

    #[regex("=|<>|<|>|<=|>=")]
    ComparisonOp,

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
        write!(f, "{self:?}")
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
