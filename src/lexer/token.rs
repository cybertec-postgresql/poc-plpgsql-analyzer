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

    #[token("editionable", ignore(case))]
    Editionable,

    #[token("noneditionable", ignore(case))]
    NonEditionable,

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

    #[token("%type", ignore(case))]
    TypeKw,

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

    #[token("not", ignore(case))]
    NotKw,

    #[regex(r"(?i)i?like")]
    LikeKw,

    #[token("(+)")]
    OracleJoinKw,

    #[regex(r"number(\s*\(\s*\d+\s*(\s*,\d+\s*)?\s*\))?", ignore(case))]
    NumberTyKw,

    // TODO: Although the correct variant (note the extra `\s*` at the beginning
    // of the matching group), this seems to match more whitespace than it should.
    // E.g. with the first one below, `p2 VARCHAR2 := ...` will produce
    // `TypeName@3..12 "VARCHAR2 "`, which obviously is _wrong_.
    // Seems to be a bug in the `logos` crate, should probably be reported to
    // the project.
    // #[regex(r"varchar2?(\s*\(\s*\d+\s*\))?", ignore(case))]
    #[regex(r"varchar2?(\(\s*\d+\s*\))?", ignore(case))]
    VarcharTyKw,

    #[regex(r"-?\d+", priority = 2)]
    Integer,

    #[regex(r"(?i)[a-z_][a-z0-9_$]*", priority = 1)]
    Ident,

    #[regex(r#""(?:[^"]|"")+""#)]
    DelimitedIdent,

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

    #[token("!")]
    Exclam,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Asterisk,

    #[token("/")]
    Slash,

    #[regex("=|<>|<|>|<=|>=")]
    ComparisonOp,

    #[token("||")]
    DoublePipe,

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

    #[test]
    fn lex_quoted_ident() {
        check(r#""读文👩🏼‍🔬""#, TokenKind::DelimitedIdent);
    }
}
