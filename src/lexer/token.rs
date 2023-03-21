// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Token definition for the [`logos`] parser.

use std::fmt;

/// Use to tokenize the input text
#[derive(logos::Logos, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    #[regex("[ \t\n\r]+")]
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

    #[token("nocopy", ignore(case))]
    NoCopyKw,

    #[token("default", ignore(case))]
    DefaultKw,

    #[token("return", ignore(case))]
    ReturnKw,

    #[token("deterministic", ignore(case))]
    DeterministicKw,

    #[token("type", ignore(case))]
    TypeKw,

    #[token("select", ignore(case))]
    SelectKw,

    #[token("insert", ignore(case))]
    InsertKw,

    #[token("values", ignore(case))]
    ValuesKw,

    #[token("into", ignore(case))]
    IntoKw,

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

    #[token("between", ignore(case))]
    BetweenKw,

    #[regex(r"(?i)i?like")]
    LikeKw,

    #[token("(+)")]
    OracleJoinKw,

    #[token("interval", ignore(case))]
    IntervalKw,

    #[token("precision", ignore(case))]
    PrecisionKw,

    #[token("binary_float", ignore(case))]
    BinaryFloatKw,

    #[token("binary_double", ignore(case))]
    BinaryDoubleKw,

    #[token("nvarchar2", ignore(case))]
    Nvarchar2Kw,

    #[token("dec", ignore(case))]
    DecKw,

    #[token("integer", ignore(case))]
    IntegerKw,

    #[token("int", ignore(case))]
    IntKw,

    #[token("numeric", ignore(case))]
    NumericKw,

    #[token("smallint", ignore(case))]
    SmallintKw,

    #[token("number", ignore(case))]
    NumberKw,

    #[token("decimal", ignore(case))]
    DecimalKw,

    #[token("double", ignore(case))]
    DoubleKw,

    #[token("float", ignore(case))]
    FloatKw,

    #[token("real", ignore(case))]
    RealKw,

    #[token("nchar", ignore(case))]
    NcharKw,

    #[token("long", ignore(case))]
    LongKw,

    #[token("char", ignore(case))]
    CharKw,

    #[token("byte", ignore(case))]
    ByteKw,

    #[token("with", ignore(case))]
    WithKw,

    #[token("local", ignore(case))]
    LocalKw,

    #[token("time", ignore(case))]
    TimeKw,

    #[token("zone", ignore(case))]
    ZoneKw,

    #[token("set", ignore(case))]
    SetKw,

    #[token("character", ignore(case))]
    CharacterKw,

    #[token("varchar2", ignore(case))]
    Varchar2Kw,

    #[token("varchar", ignore(case))]
    VarcharKw,

    #[token("raw", ignore(case))]
    RawKw,

    #[token("date", ignore(case))]
    DateKw,

    #[token("rowid", ignore(case))]
    RowidKw,

    #[token("urowid", ignore(case))]
    UrowidKw,

    #[token("timestamp", ignore(case))]
    TimestampKw,

    #[token("bfile", ignore(case))]
    BfileKw,

    #[token("blob", ignore(case))]
    BlobKw,

    #[token("clob", ignore(case))]
    ClobKw,

    #[token("nclob", ignore(case))]
    NclobKw,

    #[token("varying", ignore(case))]
    VaryingKw,

    #[token("national", ignore(case))]
    NationalKw,

    #[token("to", ignore(case))]
    ToKw,

    #[token("year", ignore(case))]
    YearKw,

    #[token("month", ignore(case))]
    MonthKw,

    #[token("day", ignore(case))]
    DayKw,

    #[token("second", ignore(case))]
    SecondKw,

    #[regex(r"-?\d+", priority = 2)]
    Integer,

    #[regex(r"(?i)[a-z_][a-z0-9_$#]*", priority = 1)]
    UnquotedIdent,

    #[regex(r#""(?:[^"]|"")+""#)]
    QuotedIdent,

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

    pub fn is_punct(self) -> bool {
        matches!(
            self,
            Self::Assign
                | Self::Asterisk
                | Self::Comma
                | Self::ComparisonOp
                | Self::DollarQuote
                | Self::Dot
                | Self::DoublePipe
                | Self::LParen
                | Self::Percentage
                | Self::RParen
                | Self::SemiColon
                | Self::Slash
        )
    }

    pub fn is_ident(self) -> bool {
        !(self.is_trivia()
            || self.is_punct()
            || matches!(
                self,
                Self::Eof | Self::Error | Self::Integer | Self::OracleJoinKw
            ))
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
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
        check(" \t \n", TokenKind::Whitespace);
    }

    #[test]
    fn lex_ident() {
        check("hello1$#", TokenKind::UnquotedIdent);
    }

    #[test]
    fn lex_quoted_ident() {
        check(r#""读文👩🏼‍🔬""#, TokenKind::QuotedIdent);
    }
}
