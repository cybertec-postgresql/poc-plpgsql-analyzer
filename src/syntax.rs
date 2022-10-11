// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

use crate::TokenKind;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

/// Examples
/// * https://blog.kiranshila.com/blog/easy_cst.md
/// * https://arzg.github.io/lang/10/
/// * https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs
///
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
    /// An identifier, e.g. secure_dml or parameter name
    Ident,
    /// A single dot
    Dot,
    /// A single comma
    Comma,
    /// A semi colon
    SemiColon,
    /// A colon token
    Colon,
    /// An Assign operator `:=`
    Assign,
    /// A single quoted literal
    QuotedLiteral,
    /// A single Param node, consisting of name & type
    Param,
    /// A node that consists of multiple parameters
    ParamList,
    /// A node that marks a full PROCEDURE block
    Procedure,
    /// A node that marks a PROCEDURE header with params
    ProcedureHeader,
    /// A node that marks a PROCEDURE body block, between `IS BEGIN` & `END;`
    ProcedureBody,
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
            TokenKind::OrReplaceKw => SyntaxKind::Keyword,
            TokenKind::BeginKw => SyntaxKind::Keyword,
            TokenKind::IsKw => SyntaxKind::Keyword,
            TokenKind::EndKw => SyntaxKind::Keyword,
            TokenKind::InKw => SyntaxKind::Keyword,
            TokenKind::OutKw => SyntaxKind::Keyword,
            TokenKind::Ident => SyntaxKind::Ident,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::SemiColon => SyntaxKind::SemiColon,
            TokenKind::Assign => SyntaxKind::Assign,
            TokenKind::QuotedLiteral => SyntaxKind::QuotedLiteral,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::Percentage => SyntaxKind::Percentage,
            TokenKind::Slash => SyntaxKind::Slash,
            TokenKind::Comment => SyntaxKind::Comment,
            TokenKind::Error => SyntaxKind::Error,
            TokenKind::Eof => unreachable!(),
        }
    }
}

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

pub type SyntaxNode = rowan::SyntaxNode<SqlProcedureLang>;
#[allow(unused)]
pub type SyntaxToken = rowan::SyntaxToken<SqlProcedureLang>;
#[allow(unused)]
pub type SyntaxElement = rowan::SyntaxElement<SqlProcedureLang>;
