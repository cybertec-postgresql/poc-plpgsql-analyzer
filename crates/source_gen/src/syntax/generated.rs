// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generated by `crates/source_gen/build.rs`, do not edit manually.

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
    #[doc = "A node containing an add_calcs_clause"]
    AddCalcsClause,
    #[doc = "An Alias for columns"]
    Alias,
    #[doc = "Logical operator AND"]
    And,
    #[doc = "A singular argument inside an argument list"]
    Argument,
    #[doc = "A list of arguments inside a `FunctionInvocation`. Made of multiple `Arguments`, separated by commas"]
    ArgumentList,
    #[doc = "Represents an arithmetic SQL operator (+, -, *, /)"]
    ArithmeticOp,
    #[doc = "An Assign operator `:=`"]
    Assign,
    #[doc = "An assignment like a=b"]
    AssignmentExpr,
    #[doc = "An asterisk `*`"]
    Asterisk,
    #[doc = "A node containing a base meas clause"]
    BaseMeasClause,
    #[doc = "A bind variable, e.g. `:OLD`"]
    BindVar,
    #[doc = "A node that marks a block"]
    Block,
    #[doc = "A node that marks an individual statement inside a block"]
    BlockStatement,
    #[doc = "A node containing a BULK COLLECT INTO clause"]
    BulkIntoClause,
    #[doc = "A node containing a calc meas clause"]
    CalcMeasClause,
    #[doc = "A colon token"]
    Colon,
    #[doc = "A single column expression, as part of an SELECT clause"]
    ColumnExpr,
    #[doc = "A single comma"]
    Comma,
    #[doc = "Inline comment starting with `--`"]
    Comment,
    #[doc = "Represents an arithmetic SQL comparison operator (=, <>, <, >, <=, >=) or other types of comparison operators of SQL (ilike, like)"]
    ComparisonOp,
    #[doc = "A concatination operator `||`"]
    Concat,
    #[doc = "The CONNECT_BY_ROOT operator"]
    ConnectByRoot,
    #[doc = "The CONNECT BY clause in selects"]
    Connect,
    #[doc = "A node that marks a full constraint"]
    Constraint,
    #[doc = "A node containing a cursor parameter declaration"]
    CursorParameterDeclaration,
    #[doc = "A node containing cursor parameter declarations"]
    CursorParameterDeclarations,
    #[doc = "A node that marks a full cursor statement"]
    CursorStmt,
    #[doc = "A node that contains a full cycle clause"]
    CycleClause,
    #[doc = "Any built-in oracle datatype"]
    Datatype,
    #[doc = "A decimal, positive, or negative"]
    Decimal,
    #[doc = "A node that marks the declare section of a block"]
    DeclareSection,
    #[doc = "A node that marks a full DELETE statement"]
    DeleteStmt,
    #[doc = "Single dollar quote `$$`"]
    DollarQuote,
    #[doc = "A single dot"]
    Dot,
    #[doc = "An error token with a cause"]
    Error,
    #[doc = "An exclamation mark `!`"]
    Exclam,
    #[doc = "A node that contains a full EXECUTE IMMEDIATE statement"]
    ExecuteImmediateStmt,
    #[doc = "Holds a generic SQL logic/arithmetic expression"]
    Expression,
    #[doc = "A node that contains a full filter clause"]
    FilterClause,
    #[doc = "A node that contains a full filter clauses"]
    FilterClauses,
    #[doc = "A node that marks a full CREATE [..] FUNCTION block"]
    Function,
    #[doc = "A node that marks a FUNCTION header with params and return type"]
    FunctionHeader,
    #[doc = "An invocation of a function, from the identifier and the opening bracket to the closing bracket"]
    FunctionInvocation,
    #[doc = "An operator in hierarchical queries"]
    HierarchicalOp,
    #[doc = "A node that marks a hierarchies clause"]
    HierarchiesClause,
    #[doc = "An identifier, either quoted or unquoted"]
    Ident,
    #[doc = "An identifier group, consisting of multiple idents"]
    IdentGroup,
    #[doc = "A node that marks a full INSERT statement"]
    InsertStmt,
    #[doc = "Any integer, positive and negative"]
    Integer,
    #[doc = "A node that contains an `INTO` clause of a SELECT statement"]
    IntoClause,
    #[doc = "A SQL keyword, e.g. `CREATE`"]
    Keyword,
    #[doc = "Represents a logical SQL operator (AND, OR, NOT)"]
    LogicOp,
    #[doc = "Left Paren"]
    LParen,
    #[doc = "A minus `-`"]
    Minus,
    #[doc = "Unary logical operator NOT"]
    Not,
    #[doc = "Logical operator OR"]
    Or,
    #[doc = "A node containing a full order by clause"]
    OrderByClause,
    #[doc = "A node that marks a full CREATE PACKAGE BODY block"]
    Package,
    #[doc = "A single Param node, consisting of name & type"]
    Param,
    #[doc = "A node that consists of multiple parameters"]
    ParamList,
    #[doc = "Percentage symbol"]
    Percentage,
    #[doc = "A plus `+`"]
    Plus,
    #[doc = "The PL/SQL unary prior operator"]
    Prior,
    #[doc = "A node that marks a full CREATE [..] PROCEDURE block"]
    Procedure,
    #[doc = "A node that marks a PROCEDURE header with params"]
    ProcedureHeader,
    #[doc = "A single quoted literal"]
    QuotedLiteral,
    #[doc = "Two dots"]
    Range,
    #[doc = "A node containing a return into clause"]
    ReturnIntoClause,
    #[doc = "A node that contains the whole RAISE statement for exceptions"]
    RaiseStmt,
    #[doc = "The root node element"]
    Root,
    #[doc = "A node containing a rowtype definition for cursors"]
    RowtypeClause,
    #[doc = "Right Paren"]
    RParen,
    #[doc = "A node containing a search clause"]
    SearchClause,
    #[doc = "A node that contains the whole SELECT clause of a query"]
    SelectClause,
    #[doc = "A node that marks a full SELECT statement"]
    SelectStmt,
    #[doc = "A semi colon"]
    Semicolon,
    #[doc = "A node containing a SET clause in an UPDATE statement"]
    SetClause,
    #[doc = "Slash char `/`"]
    Slash,
    #[doc = "A STARTS WITH clause in a SELECT statement"]
    Starts,
    #[doc = "A node containing a full subav clause"]
    SubavClause,
    #[doc = "A node containing a full subav factoring clause"]
    SubavFactoringClause,
    #[doc = "A node containing a full subquery factoring clause"]
    SubqueryFactoringClause,
    #[doc = "A text slice node"]
    Text,
    #[doc = "A node that marks a full CREATE [..] TRIGGER block"]
    Trigger,
    #[doc = "A node that marks a TRIGGER header"]
    TriggerHeader,
    #[doc = "A `%TYPE` attribute"]
    TypeAttribute,
    #[doc = "A type name"]
    TypeName,
    #[doc = "A node that marks a full UPDATE statement"]
    UpdateStmt,
    #[doc = "A node containing a using clause"]
    UsingClause,
    #[doc = "A node that marks a variable declaration as part of a function or procedure"]
    VariableDecl,
    #[doc = "A node that marks a list of variable declarations of functions and procedures"]
    VariableDeclList,
    #[doc = "A node that marks a full CREATE VIEW block"]
    View,
    #[doc = "Represent a complete `WHERE` clause expression"]
    WhereClause,
    #[doc = "Any whitespace character"]
    Whitespace,
    #[doc = "A node containing a with clause"]
    WithClause,
}
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}
impl From<TokenKind> for SyntaxKind {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Comment => SyntaxKind::Comment,
            TokenKind::Whitespace => SyntaxKind::Whitespace,
            TokenKind::DollarQuote => SyntaxKind::DollarQuote,
            TokenKind::Assign => SyntaxKind::Assign,
            TokenKind::Asterisk => SyntaxKind::Asterisk,
            TokenKind::Comma => SyntaxKind::Comma,
            TokenKind::Comparison => SyntaxKind::ComparisonOp,
            TokenKind::Dot => SyntaxKind::Dot,
            TokenKind::DoubleDot => SyntaxKind::Range,
            TokenKind::DoublePipe => SyntaxKind::Concat,
            TokenKind::Equals => SyntaxKind::ComparisonOp,
            TokenKind::Exclam => SyntaxKind::Exclam,
            TokenKind::LParen => SyntaxKind::LParen,
            TokenKind::Minus => SyntaxKind::ArithmeticOp,
            TokenKind::OracleJoin => SyntaxKind::Keyword,
            TokenKind::Percentage => SyntaxKind::Percentage,
            TokenKind::Plus => SyntaxKind::ArithmeticOp,
            TokenKind::RParen => SyntaxKind::RParen,
            TokenKind::Semicolon => SyntaxKind::Semicolon,
            TokenKind::Slash => SyntaxKind::Slash,
            TokenKind::Integer => SyntaxKind::Integer,
            TokenKind::Decimal => SyntaxKind::Decimal,
            TokenKind::UnquotedIdent => SyntaxKind::Ident,
            TokenKind::QuotedIdent => SyntaxKind::Ident,
            TokenKind::QuotedLiteral => SyntaxKind::QuotedLiteral,
            TokenKind::BindVar => SyntaxKind::BindVar,
            TokenKind::AddKw => SyntaxKind::Keyword,
            TokenKind::AfterKw => SyntaxKind::Keyword,
            TokenKind::AgentKw => SyntaxKind::Keyword,
            TokenKind::AggregateKw => SyntaxKind::Keyword,
            TokenKind::AllKw => SyntaxKind::Keyword,
            TokenKind::AllowKw => SyntaxKind::Keyword,
            TokenKind::AlterKw => SyntaxKind::Keyword,
            TokenKind::AnalyticKw => SyntaxKind::Keyword,
            TokenKind::AnalyzeKw => SyntaxKind::Keyword,
            TokenKind::AndKw => SyntaxKind::Keyword,
            TokenKind::AnnotationsKw => SyntaxKind::Keyword,
            TokenKind::AnyschemaKw => SyntaxKind::Keyword,
            TokenKind::ArrayKw => SyntaxKind::Keyword,
            TokenKind::AsKw => SyntaxKind::Keyword,
            TokenKind::AscKw => SyntaxKind::Keyword,
            TokenKind::AssociateKw => SyntaxKind::Keyword,
            TokenKind::AuditKw => SyntaxKind::Keyword,
            TokenKind::BeforeKw => SyntaxKind::Keyword,
            TokenKind::BeginKw => SyntaxKind::Keyword,
            TokenKind::BequeathKw => SyntaxKind::Keyword,
            TokenKind::BetweenKw => SyntaxKind::Keyword,
            TokenKind::BfileKw => SyntaxKind::Keyword,
            TokenKind::BinaryKw => SyntaxKind::Keyword,
            TokenKind::BinaryDoubleKw => SyntaxKind::Keyword,
            TokenKind::BinaryFloatKw => SyntaxKind::Keyword,
            TokenKind::BinaryIntegerKw => SyntaxKind::Keyword,
            TokenKind::BlobKw => SyntaxKind::Keyword,
            TokenKind::BodyKw => SyntaxKind::Keyword,
            TokenKind::BreadthKw => SyntaxKind::Keyword,
            TokenKind::BulkKw => SyntaxKind::Keyword,
            TokenKind::ByKw => SyntaxKind::Keyword,
            TokenKind::ByteKw => SyntaxKind::Keyword,
            TokenKind::CallKw => SyntaxKind::Keyword,
            TokenKind::CascadeKw => SyntaxKind::Keyword,
            TokenKind::CKw => SyntaxKind::Keyword,
            TokenKind::CharKw => SyntaxKind::Keyword,
            TokenKind::CharacterKw => SyntaxKind::Keyword,
            TokenKind::CharsetformKw => SyntaxKind::Keyword,
            TokenKind::CharsetidKw => SyntaxKind::Keyword,
            TokenKind::CheckKw => SyntaxKind::Keyword,
            TokenKind::ClobKw => SyntaxKind::Keyword,
            TokenKind::CloneKw => SyntaxKind::Keyword,
            TokenKind::CollationKw => SyntaxKind::Keyword,
            TokenKind::CollectKw => SyntaxKind::Keyword,
            TokenKind::CommentKw => SyntaxKind::Keyword,
            TokenKind::ConnectKw => SyntaxKind::Keyword,
            TokenKind::ConnectByRootKw => SyntaxKind::Keyword,
            TokenKind::ConstantKw => SyntaxKind::Keyword,
            TokenKind::ConstraintKw => SyntaxKind::Keyword,
            TokenKind::ContainerKw => SyntaxKind::Keyword,
            TokenKind::ContainerMapKw => SyntaxKind::Keyword,
            TokenKind::ContainersDefaultKw => SyntaxKind::Keyword,
            TokenKind::ContextKw => SyntaxKind::Keyword,
            TokenKind::CreateKw => SyntaxKind::Keyword,
            TokenKind::CrosseditionKw => SyntaxKind::Keyword,
            TokenKind::CurrentUserKw => SyntaxKind::Keyword,
            TokenKind::CursorKw => SyntaxKind::Keyword,
            TokenKind::CycleKw => SyntaxKind::Keyword,
            TokenKind::DataKw => SyntaxKind::Keyword,
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
            TokenKind::DefinerKw => SyntaxKind::Keyword,
            TokenKind::DeleteKw => SyntaxKind::Keyword,
            TokenKind::DepthKw => SyntaxKind::Keyword,
            TokenKind::DescKw => SyntaxKind::Keyword,
            TokenKind::DeterministicKw => SyntaxKind::Keyword,
            TokenKind::DisableKw => SyntaxKind::Keyword,
            TokenKind::DisallowKw => SyntaxKind::Keyword,
            TokenKind::DisassociateKw => SyntaxKind::Keyword,
            TokenKind::DoubleKw => SyntaxKind::Keyword,
            TokenKind::DropKw => SyntaxKind::Keyword,
            TokenKind::DurationKw => SyntaxKind::Keyword,
            TokenKind::EachKw => SyntaxKind::Keyword,
            TokenKind::EditionableKw => SyntaxKind::Keyword,
            TokenKind::EditioningKw => SyntaxKind::Keyword,
            TokenKind::ElementKw => SyntaxKind::Keyword,
            TokenKind::ElseKw => SyntaxKind::Keyword,
            TokenKind::ElsifKw => SyntaxKind::Keyword,
            TokenKind::EnableKw => SyntaxKind::Keyword,
            TokenKind::EndKw => SyntaxKind::Keyword,
            TokenKind::EnvKw => SyntaxKind::Keyword,
            TokenKind::ExceptionKw => SyntaxKind::Keyword,
            TokenKind::ExceptionsKw => SyntaxKind::Keyword,
            TokenKind::ExecuteKw => SyntaxKind::Keyword,
            TokenKind::ExistsKw => SyntaxKind::Keyword,
            TokenKind::ExtendedKw => SyntaxKind::Keyword,
            TokenKind::ExternalKw => SyntaxKind::Keyword,
            TokenKind::FactKw => SyntaxKind::Keyword,
            TokenKind::FilterKw => SyntaxKind::Keyword,
            TokenKind::FirstKw => SyntaxKind::Keyword,
            TokenKind::FloatKw => SyntaxKind::Keyword,
            TokenKind::FollowsKw => SyntaxKind::Keyword,
            TokenKind::ForKw => SyntaxKind::Keyword,
            TokenKind::ForceKw => SyntaxKind::Keyword,
            TokenKind::ForeignKw => SyntaxKind::Keyword,
            TokenKind::ForwardKw => SyntaxKind::Keyword,
            TokenKind::FromKw => SyntaxKind::Keyword,
            TokenKind::FunctionKw => SyntaxKind::Keyword,
            TokenKind::GrantKw => SyntaxKind::Keyword,
            TokenKind::HierarchiesKw => SyntaxKind::Keyword,
            TokenKind::IdKw => SyntaxKind::Keyword,
            TokenKind::IdentifierKw => SyntaxKind::Keyword,
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
            TokenKind::InvisibleKw => SyntaxKind::Keyword,
            TokenKind::IsKw => SyntaxKind::Keyword,
            TokenKind::JavaKw => SyntaxKind::Keyword,
            TokenKind::KeyKw => SyntaxKind::Keyword,
            TokenKind::LanguageKw => SyntaxKind::Keyword,
            TokenKind::LargeKw => SyntaxKind::Keyword,
            TokenKind::LastKw => SyntaxKind::Keyword,
            TokenKind::LengthKw => SyntaxKind::Keyword,
            TokenKind::LibraryKw => SyntaxKind::Keyword,
            TokenKind::LikeKw => SyntaxKind::ComparisonOp,
            TokenKind::LobsKw => SyntaxKind::Keyword,
            TokenKind::LocalKw => SyntaxKind::Keyword,
            TokenKind::LogoffKw => SyntaxKind::Keyword,
            TokenKind::LogonKw => SyntaxKind::Keyword,
            TokenKind::LongKw => SyntaxKind::Keyword,
            TokenKind::MaxlenKw => SyntaxKind::Keyword,
            TokenKind::MeasuresKw => SyntaxKind::Keyword,
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
            TokenKind::NocycleKw => SyntaxKind::Keyword,
            TokenKind::NoneKw => SyntaxKind::Keyword,
            TokenKind::NoneditionableKw => SyntaxKind::Keyword,
            TokenKind::NonschemaKw => SyntaxKind::Keyword,
            TokenKind::NoprecheckKw => SyntaxKind::Keyword,
            TokenKind::NorelyKw => SyntaxKind::Keyword,
            TokenKind::NotKw => SyntaxKind::Keyword,
            TokenKind::NovalidateKw => SyntaxKind::Keyword,
            TokenKind::NullKw => SyntaxKind::Keyword,
            TokenKind::NullsKw => SyntaxKind::Keyword,
            TokenKind::NumberKw => SyntaxKind::Keyword,
            TokenKind::NumericKw => SyntaxKind::Keyword,
            TokenKind::Nvarchar2Kw => SyntaxKind::Keyword,
            TokenKind::ObjectKw => SyntaxKind::Keyword,
            TokenKind::OfKw => SyntaxKind::Keyword,
            TokenKind::OldKw => SyntaxKind::Keyword,
            TokenKind::OnKw => SyntaxKind::Keyword,
            TokenKind::OnlyKw => SyntaxKind::Keyword,
            TokenKind::OptionKw => SyntaxKind::Keyword,
            TokenKind::OrKw => SyntaxKind::Keyword,
            TokenKind::OrderKw => SyntaxKind::Keyword,
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
            TokenKind::PriorKw => SyntaxKind::Keyword,
            TokenKind::PrimaryKw => SyntaxKind::Keyword,
            TokenKind::ProcedureKw => SyntaxKind::Keyword,
            TokenKind::RangeKw => SyntaxKind::Keyword,
            TokenKind::RaiseKw => SyntaxKind::Keyword,
            TokenKind::RawKw => SyntaxKind::Keyword,
            TokenKind::ReadKw => SyntaxKind::Keyword,
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
            TokenKind::SearchKw => SyntaxKind::Keyword,
            TokenKind::SecondKw => SyntaxKind::Keyword,
            TokenKind::SelectKw => SyntaxKind::Keyword,
            TokenKind::SelfKw => SyntaxKind::Keyword,
            TokenKind::ServererrorKw => SyntaxKind::Keyword,
            TokenKind::SetKw => SyntaxKind::Keyword,
            TokenKind::SharingKw => SyntaxKind::Keyword,
            TokenKind::ShutdownKw => SyntaxKind::Keyword,
            TokenKind::SiblingsKw => SyntaxKind::Keyword,
            TokenKind::SignatureKw => SyntaxKind::Keyword,
            TokenKind::SmallintKw => SyntaxKind::Keyword,
            TokenKind::StartsKw => SyntaxKind::Keyword,
            TokenKind::StartupKw => SyntaxKind::Keyword,
            TokenKind::StatisticsKw => SyntaxKind::Keyword,
            TokenKind::StoreKw => SyntaxKind::Keyword,
            TokenKind::StringKw => SyntaxKind::Keyword,
            TokenKind::StructKw => SyntaxKind::Keyword,
            TokenKind::SubtypeKw => SyntaxKind::Keyword,
            TokenKind::SuspendKw => SyntaxKind::Keyword,
            TokenKind::TableKw => SyntaxKind::Keyword,
            TokenKind::TablesKw => SyntaxKind::Keyword,
            TokenKind::TdoKw => SyntaxKind::Keyword,
            TokenKind::ThenKw => SyntaxKind::Keyword,
            TokenKind::TimeKw => SyntaxKind::Keyword,
            TokenKind::TimestampKw => SyntaxKind::Keyword,
            TokenKind::ToKw => SyntaxKind::Keyword,
            TokenKind::TriggerKw => SyntaxKind::Keyword,
            TokenKind::TruncateKw => SyntaxKind::Keyword,
            TokenKind::TypeKw => SyntaxKind::Keyword,
            TokenKind::UnderKw => SyntaxKind::Keyword,
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
            TokenKind::VarraysKw => SyntaxKind::Keyword,
            TokenKind::VaryingKw => SyntaxKind::Keyword,
            TokenKind::ViewKw => SyntaxKind::Keyword,
            TokenKind::VisibleKw => SyntaxKind::Keyword,
            TokenKind::WhenKw => SyntaxKind::Keyword,
            TokenKind::WhereKw => SyntaxKind::Keyword,
            TokenKind::WithKw => SyntaxKind::Keyword,
            TokenKind::XmlschemaKw => SyntaxKind::Keyword,
            TokenKind::XmltypeKw => SyntaxKind::Keyword,
            TokenKind::YearKw => SyntaxKind::Keyword,
            TokenKind::ZoneKw => SyntaxKind::Keyword,
            TokenKind::Error => SyntaxKind::Error,
            TokenKind::Eof => unreachable!(),
        }
    }
}
