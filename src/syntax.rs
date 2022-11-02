// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements a syntax-level representation of the input.

use crate::lexer::TokenKind;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

/// Represents all possible kind of syntax items the parser can process.
///
/// Examples
/// * <https://blog.kiranshila.com/blog/easy_cst.md>
/// * <https://arzg.github.io/lang/10/>
/// * <https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum SyntaxKind {
    /// Left Paren
    LParen,
    /// Right Paren
    RParen,
    /// Percentage symbol
    Percentage,
    /// Slash char '/'
    Slash,
    /// Inline comment starting with '--'
    Comment,
    /// Any whitespace character
    Whitespace,
    /// A SQL keyword, e.g. "CREATE"
    Keyword,
    /// An identifier, e.g. secure_dml or parameter name, potentially
    /// schema-qualified
    Ident,
    /// A type name
    TypeName,
    /// A single dot
    Dot,
    /// A single comma
    Comma,
    /// A semi colon
    SemiColon,
    /// An asterisk `*`
    Asterisk,
    /// A colon token
    Colon,
    /// An Assign operator `:=`
    Assign,
    /// Any integer, positive and negative
    Integer,
    /// Single dollar quote `$$`
    DollarQuote,
    /// A single quoted literal
    QuotedLiteral,
    /// A single Param node, consisting of name & type
    Param,
    /// A node that consists of multiple parameters
    ParamList,
    /// A node that marks a full CREATE [..] PROCEDURE block
    Procedure,
    /// A node that marks a PROCEDURE header with params
    ProcedureHeader,
    /// A node that marks the `IS` or `AS $$` prologue of a procedure
    ProcedurePrologue,
    /// A node that marks a PROCEDURE body block, between `{IS,AS} BEGIN` & `END;`
    ProcedureBody,
    /// A node that marks a full CREATE [..] FUNCTION block
    Function,
    /// A node that marks a FUNCTION header with params and return type
    FunctionHeader,
    /// A node that marks a FUNCTION body block, between `{IS,AS} BEGIN` & `END;`
    FunctionBody,
    /// A node that marks a full SELECT statement
    SelectStmt,
    /// A single column expression
    ColumnExpr,
    /// A node that consists of multiple column expressions
    ColumnExprList,
    /// Represent a complete `WHERE` clause expression
    WhereClause,
    /// Holds a generic SQL logic/arithmetic expression
    Expression,
    /// Represents an arithmetic SQL comparison operator (=, <>, <, >, <=, >=)
    /// or other types of comparison operators of SQL (ilike, like)
    ComparisonOp,
    /// A text slice node
    Text,
    /// An error token with a cause
    Error,
    /// The root node element
    Root,
}

impl SyntaxKind {
    /// Returns true when the [`SyntaxKind`] are not syntactically important.
    #[allow(unused)]
    pub(crate) fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }

    /// Returns true if the [`SyntaxKind`] is a keyword
    #[allow(unused)]
    pub(crate) fn is_keyword(self) -> bool {
        matches!(self, SyntaxKind::Keyword,)
    }

    #[allow(unused)]
    pub(crate) fn is_punct(self) -> bool {
        matches!(
            self,
            Self::LParen
                | Self::RParen
                | Self::Percentage
                | Self::Slash
                | Self::Dot
                | Self::Comma
                | Self::SemiColon
                | Self::Colon
                | Self::Asterisk
                | Self::ComparisonOp
        )
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

impl From<TokenKind> for SyntaxKind {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Whitespace => SyntaxKind::Whitespace,
            TokenKind::CreateKw => SyntaxKind::Keyword,
            TokenKind::ProcedureKw => SyntaxKind::Keyword,
            TokenKind::FunctionKw => SyntaxKind::Keyword,
            TokenKind::ReplaceKw => SyntaxKind::Keyword,
            TokenKind::BeginKw => SyntaxKind::Keyword,
            TokenKind::IsKw => SyntaxKind::Keyword,
            TokenKind::AsKw => SyntaxKind::Keyword,
            TokenKind::DollarQuote => SyntaxKind::DollarQuote,
            TokenKind::EndKw => SyntaxKind::Keyword,
            TokenKind::OutKw => SyntaxKind::Keyword,
            TokenKind::InKw => SyntaxKind::Keyword,
            TokenKind::ReturnKw => SyntaxKind::Keyword,
            TokenKind::DeterministicKw => SyntaxKind::Keyword,
            TokenKind::TypeKw => SyntaxKind::Keyword,
            TokenKind::NumberKw => SyntaxKind::TypeName,
            TokenKind::SelectKw => SyntaxKind::Keyword,
            TokenKind::FromKw => SyntaxKind::Keyword,
            TokenKind::WhereKw => SyntaxKind::Keyword,
            TokenKind::AndKw => SyntaxKind::Keyword,
            TokenKind::OrKw => SyntaxKind::Keyword,
            TokenKind::LikeKw => SyntaxKind::ComparisonOp,
            TokenKind::OracleJoinKw => SyntaxKind::Keyword,
            TokenKind::Integer => SyntaxKind::Integer,
            TokenKind::Ident => SyntaxKind::Ident,
            TokenKind::QuotedLiteral => SyntaxKind::QuotedLiteral,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::SemiColon => SyntaxKind::SemiColon,
            TokenKind::Asterisk => SyntaxKind::Asterisk,
            TokenKind::Assign => SyntaxKind::Assign,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::Percentage => SyntaxKind::Percentage,
            TokenKind::Slash => SyntaxKind::Slash,
            TokenKind::ComparisonOp => SyntaxKind::ComparisonOp,
            TokenKind::Comment => SyntaxKind::Comment,
            TokenKind::Error => SyntaxKind::Error,
            TokenKind::Eof => unreachable!(),
        }
    }
}

/// Dummy type for our PL/SQL language definition, for use with rowan.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SqlProcedureLang {}

impl rowan::Language for SqlProcedureLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}

/// Typed [`SyntaxNode`] with our [`SqlProcedureLang`] language definition.
pub type SyntaxNode = rowan::SyntaxNode<SqlProcedureLang>;
/// Typed [`SyntaxToken`] with our [`SqlProcedureLang`] language definition.
pub type SyntaxToken = rowan::SyntaxToken<SqlProcedureLang>;
/// Typed [`SyntaxElement`] with our [`SqlProcedureLang`] language definition.
#[allow(unused)]
pub type SyntaxElement = rowan::SyntaxElement<SqlProcedureLang>;
