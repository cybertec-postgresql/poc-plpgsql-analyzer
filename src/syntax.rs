// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements a syntax-level representation of the input.

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::lexer::TokenKind;

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
    /// An exclamation mark `!`
    Exclam,
    /// A plus `+`
    Plus,
    /// A minus `-`
    Minus,
    /// An asterisk `*`
    Asterisk,
    /// Slash char '/'
    Slash,
    /// Logical operator AND
    And,
    /// Logical operator OR
    Or,
    /// Unary logical operator NOT
    Not,
    /// Inline comment starting with '--'
    Comment,
    /// Any whitespace character
    Whitespace,
    /// A SQL keyword, e.g. "CREATE"
    Keyword,
    /// An identifier group, consisting of multiple idents
    IdentGroup,
    /// An identifier, either quoted or unquoted
    Ident,
    /// A type name
    TypeName,
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
    /// A concatination operator `||`
    Concat,
    /// Any built-in oracle datatype
    Datatype,
    /// A `%TYPE` attribute
    TypeAttribute,
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
    /// A node that marks a block
    Block,
    /// A node that marks an individual statement inside a block
    BlockStatement,
    /// A node that marks the declare section of a block
    DeclareSection,
    /// An invocation of a function, from the identifier and the opening bracket to the closing bracket
    FunctionInvocation,
    /// A list of arguments inside a `FunctionInvocation`. Made of multiple `Arguments`, separated by commas
    ArgumentList,
    /// A singular argument inside an argument list
    Argument,
    /// A node that marks a full CREATE [..] FUNCTION block
    Function,
    /// A node that marks a FUNCTION header with params and return type
    FunctionHeader,
    /// A node that marks a full SELECT statement
    SelectStmt,
    /// A node that marks a full INSERT statement
    InsertStmt,
    /// A single column expression, as part of an SELECT clause
    ColumnExpr,
    /// A node that contains the whole SELECT clause of a query
    SelectClause,
    /// A node that contains an `INTO` clause of a SELECT statement
    IntoClause,
    /// Represent a complete `WHERE` clause expression
    WhereClause,
    /// A node that marks a variable declaration as part of a function or
    /// procedure
    VariableDecl,
    /// A node that marks a list of variable declarations of functions and
    /// procedures
    VariableDeclList,
    /// Holds a generic SQL logic/arithmetic expression
    Expression,
    /// Represents an arithmetic SQL operator (+, -, *, /)
    ArithmeticOp,
    /// Represents an arithmetic SQL comparison operator (=, <>, <, >, <=, >=)
    /// or other types of comparison operators of SQL (ilike, like)
    ComparisonOp,
    /// Represents a logical SQL operator (AND, OR, NOT)
    LogicOp,
    /// A text slice node
    Text,
    /// An error token with a cause
    Error,
    /// The root node element
    Root,
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
            TokenKind::Editionable => SyntaxKind::Keyword,
            TokenKind::NonEditionable => SyntaxKind::Keyword,
            TokenKind::ProcedureKw => SyntaxKind::Keyword,
            TokenKind::FunctionKw => SyntaxKind::Keyword,
            TokenKind::ReplaceKw => SyntaxKind::Keyword,
            TokenKind::DeclareKw => SyntaxKind::Keyword,
            TokenKind::BeginKw => SyntaxKind::Keyword,
            TokenKind::NullKw => SyntaxKind::Keyword,
            TokenKind::IfKw => SyntaxKind::Keyword,
            TokenKind::ThenKw => SyntaxKind::Keyword,
            TokenKind::ElsifKw => SyntaxKind::Keyword,
            TokenKind::ElseKw => SyntaxKind::Keyword,
            TokenKind::IsKw => SyntaxKind::Keyword,
            TokenKind::AsKw => SyntaxKind::Keyword,
            TokenKind::DollarQuote => SyntaxKind::DollarQuote,
            TokenKind::EndKw => SyntaxKind::Keyword,
            TokenKind::OutKw => SyntaxKind::Keyword,
            TokenKind::NoCopyKw => SyntaxKind::Keyword,
            TokenKind::DefaultKw => SyntaxKind::Keyword,
            TokenKind::InKw => SyntaxKind::Keyword,
            TokenKind::ReturnKw => SyntaxKind::Keyword,
            TokenKind::DeterministicKw => SyntaxKind::Keyword,
            TokenKind::TypeKw => SyntaxKind::Keyword,
            TokenKind::SelectKw => SyntaxKind::Keyword,
            TokenKind::InsertKw => SyntaxKind::Keyword,
            TokenKind::ValuesKw => SyntaxKind::Keyword,
            TokenKind::IntoKw => SyntaxKind::Keyword,
            TokenKind::FromKw => SyntaxKind::Keyword,
            TokenKind::WhereKw => SyntaxKind::Keyword,
            TokenKind::AndKw => SyntaxKind::Keyword,
            TokenKind::OrKw => SyntaxKind::Keyword,
            TokenKind::NotKw => SyntaxKind::Keyword,
            TokenKind::BetweenKw => SyntaxKind::Keyword,
            TokenKind::LikeKw => SyntaxKind::ComparisonOp,
            TokenKind::OracleJoinKw => SyntaxKind::Keyword,
            TokenKind::IntervalKw => SyntaxKind::Keyword,
            TokenKind::PrecisionKw => SyntaxKind::Keyword,
            TokenKind::BinaryFloatKw => SyntaxKind::Keyword,
            TokenKind::BinaryDoubleKw => SyntaxKind::Keyword,
            TokenKind::Nvarchar2Kw => SyntaxKind::Keyword,
            TokenKind::DecKw => SyntaxKind::Keyword,
            TokenKind::IntegerKw => SyntaxKind::Keyword,
            TokenKind::IntKw => SyntaxKind::Keyword,
            TokenKind::NumericKw => SyntaxKind::Keyword,
            TokenKind::SmallintKw => SyntaxKind::Keyword,
            TokenKind::NumberKw => SyntaxKind::Keyword,
            TokenKind::DecimalKw => SyntaxKind::Keyword,
            TokenKind::DoubleKw => SyntaxKind::Keyword,
            TokenKind::FloatKw => SyntaxKind::Keyword,
            TokenKind::RealKw => SyntaxKind::Keyword,
            TokenKind::NcharKw => SyntaxKind::Keyword,
            TokenKind::LongKw => SyntaxKind::Keyword,
            TokenKind::CharKw => SyntaxKind::Keyword,
            TokenKind::ByteKw => SyntaxKind::Keyword,
            TokenKind::WithKw => SyntaxKind::Keyword,
            TokenKind::LocalKw => SyntaxKind::Keyword,
            TokenKind::TimeKw => SyntaxKind::Keyword,
            TokenKind::ZoneKw => SyntaxKind::Keyword,
            TokenKind::SetKw => SyntaxKind::Keyword,
            TokenKind::CharacterKw => SyntaxKind::Keyword,
            TokenKind::Varchar2Kw => SyntaxKind::Keyword,
            TokenKind::VarcharKw => SyntaxKind::Keyword,
            TokenKind::RawKw => SyntaxKind::Keyword,
            TokenKind::DateKw => SyntaxKind::Keyword,
            TokenKind::RowidKw => SyntaxKind::Keyword,
            TokenKind::UrowidKw => SyntaxKind::Keyword,
            TokenKind::TimestampKw => SyntaxKind::Keyword,
            TokenKind::BfileKw => SyntaxKind::Keyword,
            TokenKind::BlobKw => SyntaxKind::Keyword,
            TokenKind::ClobKw => SyntaxKind::Keyword,
            TokenKind::NclobKw => SyntaxKind::Keyword,
            TokenKind::VaryingKw => SyntaxKind::Keyword,
            TokenKind::NationalKw => SyntaxKind::Keyword,
            TokenKind::ToKw => SyntaxKind::Keyword,
            TokenKind::YearKw => SyntaxKind::Keyword,
            TokenKind::MonthKw => SyntaxKind::Keyword,
            TokenKind::DayKw => SyntaxKind::Keyword,
            TokenKind::SecondKw => SyntaxKind::Keyword,
            TokenKind::Integer => SyntaxKind::Integer,
            TokenKind::UnquotedIdent => SyntaxKind::Ident,
            TokenKind::QuotedIdent => SyntaxKind::Ident,
            TokenKind::QuotedLiteral => SyntaxKind::QuotedLiteral,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::SemiColon => SyntaxKind::SemiColon,
            TokenKind::Assign => SyntaxKind::Assign,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::Percentage => SyntaxKind::Percentage,
            TokenKind::Exclam => SyntaxKind::Exclam,
            // Used in `SELECT *` or as an arithmetic op
            // Mapping to arithmetic op happens in `grammar/expressions.rs`
            TokenKind::Asterisk => SyntaxKind::Asterisk,
            TokenKind::Slash => SyntaxKind::Slash,
            TokenKind::Plus => SyntaxKind::ArithmeticOp,
            TokenKind::Minus => SyntaxKind::ArithmeticOp,
            TokenKind::ComparisonOp => SyntaxKind::ComparisonOp,
            TokenKind::DoublePipe => SyntaxKind::Concat,
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
