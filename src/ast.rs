use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::lexer::TokenKind;

/// Examples
/// * https://blog.kiranshila.com/blog/easy_cst.md
/// * https://arzg.github.io/lang/10/
/// * https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum SyntaxKind {
    /// Left Paren
    LParen = 0,
    /// Right Paren
    RParen,
    /// Inline comment starting with '--'
    Comment,
    /// Any whitespace character
    Whitespace,
    /// A SQL keyword, e.g. "CREATE"
    Keyword,
    /// An identifier, e.g. secure_dml
    Ident,
    /// A single dot
    Dot,
    /// A single comma
    Comma,
    /// A semi colon
    SemiColon,
    /// A single Param node, consisting of name & type
    Param,
    /// A node that contains a list of [`SyntaxKind::Param`]
    ParamList,
    /// A node that represents the parameter name, contains [`SyntaxKind::Ident`]
    ParamName,
    /// A node that marks a type parameter
    ParamType,
    /// A node that marks a full PROCEDURE block
    Procedure,
    /// A node that marks a PROCEDURE header block
    ProcedureStart,
    /// A node that marks a PROCEDURE header with params
    ProcedureHeader,
    /// A node that marks a PROCEDURE body block, between `IS BEGIN` & `END;`
    ProcedureBody,
    /// A node that is not further analyzed yet, e.g. procedure body content.
    Unsupported,
    /// A text slice node
    Text,
    /// An error token with a cause
    Error,
    /// The root node element
    Root,
}

impl SyntaxKind {
    pub(crate) fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
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
            TokenKind::Ident => SyntaxKind::Ident,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::SemiColon => SyntaxKind::SemiColon,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::Comment => SyntaxKind::Comment,
            TokenKind::Error => SyntaxKind::Error,
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
pub type SyntaxToken = rowan::SyntaxToken<SqlProcedureLang>;
pub type SyntaxElement = rowan::SyntaxElement<SqlProcedureLang>;
