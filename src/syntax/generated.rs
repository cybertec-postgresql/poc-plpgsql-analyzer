// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generated by `src/sourcegen/syntax.rs`, do not edit manually.

use crate::lexer::TokenKind;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
#[doc = r" Represents all possible kind of syntax items the parser can process."]
#[doc = r""]
#[doc = r" Examples"]
#[doc = r" * <https://blog.kiranshila.com/blog/easy_cst.md>"]
#[doc = r" * <https://arzg.github.io/lang/10/>"]
#[doc = r" * <https://github.com/rust-analyzer/rowan/blob/master/examples/s_expressions.rs>"]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
#[repr(u16)]
pub enum SyntaxKind {
    #[doc = "Left Paren"]
    LParen,
    #[doc = "Right Paren"]
    RParen,
    #[doc = "Percentage symbol"]
    Percentage,
    #[doc = "An exclamation mark `!`"]
    Exclam,
    #[doc = "A bind variable, e.g. `:OLD`"]
    BindVar,
    #[doc = "A plus `+`"]
    Plus,
    #[doc = "A minus `-`"]
    Minus,
    #[doc = "An asterisk `*`"]
    Asterisk,
    #[doc = "Slash char `/`"]
    Slash,
    #[doc = "Logical operator AND"]
    And,
    #[doc = "Logical operator OR"]
    Or,
    #[doc = "Unary logical operator NOT"]
    Not,
    #[doc = "Inline comment starting with `--`"]
    Comment,
    #[doc = "Any whitespace character"]
    Whitespace,
    #[doc = "A SQL keyword, e.g. `CREATE`"]
    Keyword,
    #[doc = "An identifier group, consisting of multiple idents"]
    IdentGroup,
    #[doc = "An identifier, either quoted or unquoted"]
    Ident,
    #[doc = "A type name"]
    TypeName,
    #[doc = "A single dot"]
    Dot,
    #[doc = "Two dots"]
    Range,
    #[doc = "A single comma"]
    Comma,
    #[doc = "A semi colon"]
    Semicolon,
    #[doc = "A colon token"]
    Colon,
    #[doc = "An Assign operator `:=`"]
    Assign,
    #[doc = "A concatination operator `||`"]
    Concat,
    #[doc = "Any built-in oracle datatype"]
    Datatype,
    #[doc = "A `%TYPE` attribute"]
    TypeAttribute,
    #[doc = "Any integer, positive and negative"]
    Integer,
    #[doc = "Single dollar quote `$$`"]
    DollarQuote,
    #[doc = "A single quoted literal"]
    QuotedLiteral,
    #[doc = "A single Param node, consisting of name & type"]
    Param,
    #[doc = "A node that consists of multiple parameters"]
    ParamList,
    #[doc = "A node that marks a full CREATE [..] PROCEDURE block"]
    Procedure,
    #[doc = "A node that marks a PROCEDURE header with params"]
    ProcedureHeader,
    #[doc = "A node that marks a full CREATE [..] TRIGGER block"]
    Trigger,
    #[doc = "A node that marks a TRIGGER header"]
    TriggerHeader,
    #[doc = "A node that marks a full CREATE PACKAGE BODY block"]
    Package,
    #[doc = "A node that marks a full CREATE VIEW block"]
    View,
    #[doc = "A node that marks a full constraint"]
    Constraint,
    #[doc = "A node that marks a block"]
    Block,
    #[doc = "A node that marks an individual statement inside a block"]
    BlockStatement,
    #[doc = "A node that marks the declare section of a block"]
    DeclareSection,
    #[doc = "An invocation of a function, from the identifier and the opening bracket to the closing bracket"]
    FunctionInvocation,
    #[doc = "A list of arguments inside a `FunctionInvocation`. Made of multiple `Arguments`, separated by commas"]
    ArgumentList,
    #[doc = "A singular argument inside an argument list"]
    Argument,
    #[doc = "A node that marks a full CREATE [..] FUNCTION block"]
    Function,
    #[doc = "A node that marks a FUNCTION header with params and return type"]
    FunctionHeader,
    #[doc = "A node that marks a full SELECT statement"]
    SelectStmt,
    #[doc = "A node that marks a full INSERT statement"]
    InsertStmt,
    #[doc = "A single column expression, as part of an SELECT clause"]
    ColumnExpr,
    #[doc = "A node that contains the whole SELECT clause of a query"]
    SelectClause,
    #[doc = "A node that contains an `INTO` clause of a SELECT statement"]
    IntoClause,
    #[doc = "Represent a complete `WHERE` clause expression"]
    WhereClause,
    #[doc = "A node that marks a variable declaration as part of a function or procedure"]
    VariableDecl,
    #[doc = "A node that marks a list of variable declarations of functions and procedures"]
    VariableDeclList,
    #[doc = "Holds a generic SQL logic/arithmetic expression"]
    Expression,
    #[doc = "Represents an arithmetic SQL operator (+, -, *, /)"]
    ArithmeticOp,
    #[doc = "Represents an arithmetic SQL comparison operator (=, <>, <, >, <=, >=) or other types of comparison operators of SQL (ilike, like)"]
    ComparisonOp,
    #[doc = "Represents a logical SQL operator (AND, OR, NOT)"]
    LogicOp,
    #[doc = "A text slice node"]
    Text,
    #[doc = "An error token with a cause"]
    Error,
    #[doc = "The root node element"]
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
            TokenKind::Comment => SyntaxKind::Comment,
            TokenKind::Exclam => SyntaxKind::Exclam,
            TokenKind::DollarQuote => SyntaxKind::DollarQuote,
            TokenKind::Percentage => SyntaxKind::Percentage,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::OracleJoin => SyntaxKind::Keyword,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::Asterisk => SyntaxKind::Asterisk,
            TokenKind::Plus => SyntaxKind::ArithmeticOp,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::Minus => SyntaxKind::ArithmeticOp,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::DoubleDot => SyntaxKind::Range,
            TokenKind::Slash => SyntaxKind::Slash,
            TokenKind::Assign => SyntaxKind::Assign,
            TokenKind::Semicolon => SyntaxKind::Semicolon,
            TokenKind::Equals => SyntaxKind::ComparisonOp,
            TokenKind::Comparison => SyntaxKind::ComparisonOp,
            TokenKind::DoublePipe => SyntaxKind::Concat,
            TokenKind::Integer => SyntaxKind::Integer,
            TokenKind::UnquotedIdent => SyntaxKind::Ident,
            TokenKind::QuotedIdent => SyntaxKind::Ident,
            TokenKind::QuotedLiteral => SyntaxKind::QuotedLiteral,
            TokenKind::BindVar => SyntaxKind::BindVar,
            TokenKind::AfterKw => SyntaxKind::Keyword,
            TokenKind::AgentKw => SyntaxKind::Keyword,
            TokenKind::AlterKw => SyntaxKind::Keyword,
            TokenKind::AnalyzeKw => SyntaxKind::Keyword,
            TokenKind::AndKw => SyntaxKind::Keyword,
            TokenKind::ArrayKw => SyntaxKind::Keyword,
            TokenKind::AsKw => SyntaxKind::Keyword,
            TokenKind::AssociateKw => SyntaxKind::Keyword,
            TokenKind::AuditKw => SyntaxKind::Keyword,
            TokenKind::BeforeKw => SyntaxKind::Keyword,
            TokenKind::BeginKw => SyntaxKind::Keyword,
            TokenKind::BetweenKw => SyntaxKind::Keyword,
            TokenKind::BfileKw => SyntaxKind::Keyword,
            TokenKind::BinaryDoubleKw => SyntaxKind::Keyword,
            TokenKind::BinaryFloatKw => SyntaxKind::Keyword,
            TokenKind::BinaryIntegerKw => SyntaxKind::Keyword,
            TokenKind::BlobKw => SyntaxKind::Keyword,
            TokenKind::BodyKw => SyntaxKind::Keyword,
            TokenKind::ByKw => SyntaxKind::Keyword,
            TokenKind::ByteKw => SyntaxKind::Keyword,
            TokenKind::CKw => SyntaxKind::Keyword,
            TokenKind::CallKw => SyntaxKind::Keyword,
            TokenKind::CascadeKw => SyntaxKind::Keyword,
            TokenKind::CharKw => SyntaxKind::Keyword,
            TokenKind::CharacterKw => SyntaxKind::Keyword,
            TokenKind::CharsetformKw => SyntaxKind::Keyword,
            TokenKind::CharsetidKw => SyntaxKind::Keyword,
            TokenKind::CheckKw => SyntaxKind::Keyword,
            TokenKind::ClobKw => SyntaxKind::Keyword,
            TokenKind::CloneKw => SyntaxKind::Keyword,
            TokenKind::CommentKw => SyntaxKind::Keyword,
            TokenKind::ConstantKw => SyntaxKind::Keyword,
            TokenKind::ConstraintKw => SyntaxKind::Keyword,
            TokenKind::ContainerKw => SyntaxKind::Keyword,
            TokenKind::ContextKw => SyntaxKind::Keyword,
            TokenKind::CreateKw => SyntaxKind::Keyword,
            TokenKind::CrosseditionKw => SyntaxKind::Keyword,
            TokenKind::CursorKw => SyntaxKind::Keyword,
            TokenKind::DatabaseKw => SyntaxKind::Keyword,
            TokenKind::DateKw => SyntaxKind::Keyword,
            TokenKind::DayKw => SyntaxKind::Keyword,
            TokenKind::DbRoleChangeKw => SyntaxKind::Keyword,
            TokenKind::DdlKw => SyntaxKind::Keyword,
            TokenKind::DecKw => SyntaxKind::Keyword,
            TokenKind::DecimalKw => SyntaxKind::Keyword,
            TokenKind::DeclareKw => SyntaxKind::Keyword,
            TokenKind::DefaultKw => SyntaxKind::Keyword,
            TokenKind::DeferrableKw => SyntaxKind::Keyword,
            TokenKind::DeferredKw => SyntaxKind::Keyword,
            TokenKind::DeleteKw => SyntaxKind::Keyword,
            TokenKind::DeterministicKw => SyntaxKind::Keyword,
            TokenKind::DisableKw => SyntaxKind::Keyword,
            TokenKind::DisassociateKw => SyntaxKind::Keyword,
            TokenKind::DoubleKw => SyntaxKind::Keyword,
            TokenKind::DropKw => SyntaxKind::Keyword,
            TokenKind::DurationKw => SyntaxKind::Keyword,
            TokenKind::EachKw => SyntaxKind::Keyword,
            TokenKind::EditionableKw => SyntaxKind::Keyword,
            TokenKind::ElseKw => SyntaxKind::Keyword,
            TokenKind::ElsifKw => SyntaxKind::Keyword,
            TokenKind::EnableKw => SyntaxKind::Keyword,
            TokenKind::EndKw => SyntaxKind::Keyword,
            TokenKind::EnvKw => SyntaxKind::Keyword,
            TokenKind::ExceptionKw => SyntaxKind::Keyword,
            TokenKind::ExceptionsKw => SyntaxKind::Keyword,
            TokenKind::ExistsKw => SyntaxKind::Keyword,
            TokenKind::ExternalKw => SyntaxKind::Keyword,
            TokenKind::FloatKw => SyntaxKind::Keyword,
            TokenKind::FollowsKw => SyntaxKind::Keyword,
            TokenKind::ForKw => SyntaxKind::Keyword,
            TokenKind::ForceKw => SyntaxKind::Keyword,
            TokenKind::ForeignKw => SyntaxKind::Keyword,
            TokenKind::ForwardKw => SyntaxKind::Keyword,
            TokenKind::FromKw => SyntaxKind::Keyword,
            TokenKind::FunctionKw => SyntaxKind::Keyword,
            TokenKind::GrantKw => SyntaxKind::Keyword,
            TokenKind::IfKw => SyntaxKind::Keyword,
            TokenKind::IlikeKw => SyntaxKind::ComparisonOp,
            TokenKind::ImmediateKw => SyntaxKind::Keyword,
            TokenKind::InKw => SyntaxKind::Keyword,
            TokenKind::IndexKw => SyntaxKind::Keyword,
            TokenKind::IndicatorKw => SyntaxKind::Keyword,
            TokenKind::InitiallyKw => SyntaxKind::Keyword,
            TokenKind::InsertKw => SyntaxKind::Keyword,
            TokenKind::InsteadKw => SyntaxKind::Keyword,
            TokenKind::IntKw => SyntaxKind::Keyword,
            TokenKind::IntegerKw => SyntaxKind::Keyword,
            TokenKind::IntervalKw => SyntaxKind::Keyword,
            TokenKind::IntoKw => SyntaxKind::Keyword,
            TokenKind::IsKw => SyntaxKind::Keyword,
            TokenKind::JavaKw => SyntaxKind::Keyword,
            TokenKind::KeyKw => SyntaxKind::Keyword,
            TokenKind::LanguageKw => SyntaxKind::Keyword,
            TokenKind::LengthKw => SyntaxKind::Keyword,
            TokenKind::LibraryKw => SyntaxKind::Keyword,
            TokenKind::LikeKw => SyntaxKind::ComparisonOp,
            TokenKind::LocalKw => SyntaxKind::Keyword,
            TokenKind::LogoffKw => SyntaxKind::Keyword,
            TokenKind::LogonKw => SyntaxKind::Keyword,
            TokenKind::LongKw => SyntaxKind::Keyword,
            TokenKind::MaxlenKw => SyntaxKind::Keyword,
            TokenKind::MetadataKw => SyntaxKind::Keyword,
            TokenKind::MleKw => SyntaxKind::Keyword,
            TokenKind::ModuleKw => SyntaxKind::Keyword,
            TokenKind::MonthKw => SyntaxKind::Keyword,
            TokenKind::NameKw => SyntaxKind::Keyword,
            TokenKind::NationalKw => SyntaxKind::Keyword,
            TokenKind::NcharKw => SyntaxKind::Keyword,
            TokenKind::NclobKw => SyntaxKind::Keyword,
            TokenKind::NewKw => SyntaxKind::Keyword,
            TokenKind::NoKw => SyntaxKind::Keyword,
            TokenKind::NoauditKw => SyntaxKind::Keyword,
            TokenKind::NocopyKw => SyntaxKind::Keyword,
            TokenKind::NoneKw => SyntaxKind::Keyword,
            TokenKind::NoneditionableKw => SyntaxKind::Keyword,
            TokenKind::NoprecheckKw => SyntaxKind::Keyword,
            TokenKind::NorelyKw => SyntaxKind::Keyword,
            TokenKind::NotKw => SyntaxKind::Keyword,
            TokenKind::NovalidateKw => SyntaxKind::Keyword,
            TokenKind::NullKw => SyntaxKind::Keyword,
            TokenKind::NumberKw => SyntaxKind::Keyword,
            TokenKind::NumericKw => SyntaxKind::Keyword,
            TokenKind::Nvarchar2Kw => SyntaxKind::Keyword,
            TokenKind::OfKw => SyntaxKind::Keyword,
            TokenKind::OldKw => SyntaxKind::Keyword,
            TokenKind::OnKw => SyntaxKind::Keyword,
            TokenKind::OrKw => SyntaxKind::Keyword,
            TokenKind::OthersKw => SyntaxKind::Keyword,
            TokenKind::OutKw => SyntaxKind::Keyword,
            TokenKind::PackageKw => SyntaxKind::Keyword,
            TokenKind::ParallelEnableKw => SyntaxKind::Keyword,
            TokenKind::ParametersKw => SyntaxKind::Keyword,
            TokenKind::ParentKw => SyntaxKind::Keyword,
            TokenKind::PipelinedKw => SyntaxKind::Keyword,
            TokenKind::PlpgsqlKw => SyntaxKind::Keyword,
            TokenKind::PlsIntegerKw => SyntaxKind::Keyword,
            TokenKind::PluggableKw => SyntaxKind::Keyword,
            TokenKind::PrecedesKw => SyntaxKind::Keyword,
            TokenKind::PrecheckKw => SyntaxKind::Keyword,
            TokenKind::PrecisionKw => SyntaxKind::Keyword,
            TokenKind::PrimaryKw => SyntaxKind::Keyword,
            TokenKind::ProcedureKw => SyntaxKind::Keyword,
            TokenKind::RangeKw => SyntaxKind::Keyword,
            TokenKind::RawKw => SyntaxKind::Keyword,
            TokenKind::RealKw => SyntaxKind::Keyword,
            TokenKind::RecordKw => SyntaxKind::Keyword,
            TokenKind::RefKw => SyntaxKind::Keyword,
            TokenKind::ReferenceKw => SyntaxKind::Keyword,
            TokenKind::ReferencesKw => SyntaxKind::Keyword,
            TokenKind::ReferencingKw => SyntaxKind::Keyword,
            TokenKind::ReliesOnKw => SyntaxKind::Keyword,
            TokenKind::RelyKw => SyntaxKind::Keyword,
            TokenKind::RenameKw => SyntaxKind::Keyword,
            TokenKind::ReplaceKw => SyntaxKind::Keyword,
            TokenKind::ResultCacheKw => SyntaxKind::Keyword,
            TokenKind::ReturnKw => SyntaxKind::Keyword,
            TokenKind::ReturningKw => SyntaxKind::Keyword,
            TokenKind::ReverseKw => SyntaxKind::Keyword,
            TokenKind::RevokeKw => SyntaxKind::Keyword,
            TokenKind::RowKw => SyntaxKind::Keyword,
            TokenKind::RowidKw => SyntaxKind::Keyword,
            TokenKind::RowtypeKw => SyntaxKind::Keyword,
            TokenKind::SchemaKw => SyntaxKind::Keyword,
            TokenKind::ScopeKw => SyntaxKind::Keyword,
            TokenKind::SecondKw => SyntaxKind::Keyword,
            TokenKind::SelectKw => SyntaxKind::Keyword,
            TokenKind::SelfKw => SyntaxKind::Keyword,
            TokenKind::ServererrorKw => SyntaxKind::Keyword,
            TokenKind::SetKw => SyntaxKind::Keyword,
            TokenKind::SharingKw => SyntaxKind::Keyword,
            TokenKind::ShutdownKw => SyntaxKind::Keyword,
            TokenKind::SignatureKw => SyntaxKind::Keyword,
            TokenKind::SmallintKw => SyntaxKind::Keyword,
            TokenKind::StartupKw => SyntaxKind::Keyword,
            TokenKind::StatisticsKw => SyntaxKind::Keyword,
            TokenKind::StringKw => SyntaxKind::Keyword,
            TokenKind::StructKw => SyntaxKind::Keyword,
            TokenKind::SubtypeKw => SyntaxKind::Keyword,
            TokenKind::SuspendKw => SyntaxKind::Keyword,
            TokenKind::TableKw => SyntaxKind::Keyword,
            TokenKind::TdoKw => SyntaxKind::Keyword,
            TokenKind::ThenKw => SyntaxKind::Keyword,
            TokenKind::TimeKw => SyntaxKind::Keyword,
            TokenKind::TimestampKw => SyntaxKind::Keyword,
            TokenKind::ToKw => SyntaxKind::Keyword,
            TokenKind::TriggerKw => SyntaxKind::Keyword,
            TokenKind::TruncateKw => SyntaxKind::Keyword,
            TokenKind::TypeKw => SyntaxKind::Keyword,
            TokenKind::UniqueKw => SyntaxKind::Keyword,
            TokenKind::UnplugKw => SyntaxKind::Keyword,
            TokenKind::UpdateKw => SyntaxKind::Keyword,
            TokenKind::UrowidKw => SyntaxKind::Keyword,
            TokenKind::UsingKw => SyntaxKind::Keyword,
            TokenKind::ValidateKw => SyntaxKind::Keyword,
            TokenKind::ValuesKw => SyntaxKind::Keyword,
            TokenKind::VarcharKw => SyntaxKind::Keyword,
            TokenKind::Varchar2Kw => SyntaxKind::Keyword,
            TokenKind::VarrayKw => SyntaxKind::Keyword,
            TokenKind::VaryingKw => SyntaxKind::Keyword,
            TokenKind::ViewKw => SyntaxKind::Keyword,
            TokenKind::WhenKw => SyntaxKind::Keyword,
            TokenKind::WhereKw => SyntaxKind::Keyword,
            TokenKind::WithKw => SyntaxKind::Keyword,
            TokenKind::YearKw => SyntaxKind::Keyword,
            TokenKind::ZoneKw => SyntaxKind::Keyword,
            TokenKind::Error => SyntaxKind::Error,
            TokenKind::Eof => unreachable!(),
        }
    }
}
