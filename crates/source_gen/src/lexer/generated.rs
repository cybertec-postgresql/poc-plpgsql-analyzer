// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Generated by `crates/source_gen/build.rs`, do not edit manually.

#[derive(logos :: Logos, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    #[regex("--.*")]
    Comment,
    #[regex("[ \t\n\r]+")]
    Whitespace,
    #[token("$$", ignore(case))]
    DollarQuote,
    #[token(":=", ignore(case))]
    Assign,
    #[token("*", ignore(case))]
    Asterisk,
    #[token(",", ignore(case))]
    Comma,
    #[regex("<>|<|>|<=|>=")]
    Comparison,
    #[token(".", ignore(case))]
    Dot,
    #[token("..", ignore(case))]
    DoubleDot,
    #[token("||", ignore(case))]
    DoublePipe,
    #[token("=", ignore(case))]
    Equals,
    #[token("!", ignore(case))]
    Exclam,
    #[token("(", ignore(case))]
    LParen,
    #[token("-", ignore(case))]
    Minus,
    #[token("(+)", ignore(case))]
    OracleJoin,
    #[token("%", ignore(case))]
    Percentage,
    #[token("+", ignore(case))]
    Plus,
    #[token(")", ignore(case))]
    RParen,
    #[token(";", ignore(case))]
    Semicolon,
    #[token("/", ignore(case))]
    Slash,
    #[regex("-?\\d+", priority = 2)]
    Integer,
    #[regex("-?(\\d+\\.\\d*|\\d*\\.\\d+)", priority = 2)]
    Decimal,
    #[regex("(?i)[a-z_][a-z0-9_$#]*", priority = 1)]
    UnquotedIdent,
    #[regex("\"(?:[^\"]|\"\")+\"")]
    QuotedIdent,
    #[regex("'[^']*'")]
    QuotedLiteral,
    #[regex("(?i):[a-z][a-z0-9_]*")]
    BindVar,
    #[token("add", ignore(case))]
    AddKw,
    #[token("after", ignore(case))]
    AfterKw,
    #[token("agent", ignore(case))]
    AgentKw,
    #[token("all", ignore(case))]
    AllKw,
    #[token("allow", ignore(case))]
    AllowKw,
    #[token("alter", ignore(case))]
    AlterKw,
    #[token("analyze", ignore(case))]
    AnalyzeKw,
    #[token("and", ignore(case))]
    AndKw,
    #[token("annotations", ignore(case))]
    AnnotationsKw,
    #[token("anyschema", ignore(case))]
    AnyschemaKw,
    #[token("array", ignore(case))]
    ArrayKw,
    #[token("as", ignore(case))]
    AsKw,
    #[token("asc", ignore(case))]
    AscKw,
    #[token("associate", ignore(case))]
    AssociateKw,
    #[token("audit", ignore(case))]
    AuditKw,
    #[token("before", ignore(case))]
    BeforeKw,
    #[token("begin", ignore(case))]
    BeginKw,
    #[token("bequeath", ignore(case))]
    BequeathKw,
    #[token("between", ignore(case))]
    BetweenKw,
    #[token("bfile", ignore(case))]
    BfileKw,
    #[token("binary", ignore(case))]
    BinaryKw,
    #[token("binary_double", ignore(case))]
    BinaryDoubleKw,
    #[token("binary_float", ignore(case))]
    BinaryFloatKw,
    #[token("binary_integer", ignore(case))]
    BinaryIntegerKw,
    #[token("blob", ignore(case))]
    BlobKw,
    #[token("body", ignore(case))]
    BodyKw,
    #[token("bulk", ignore(case))]
    BulkKw,
    #[token("by", ignore(case))]
    ByKw,
    #[token("byte", ignore(case))]
    ByteKw,
    #[token("call", ignore(case))]
    CallKw,
    #[token("cascade", ignore(case))]
    CascadeKw,
    #[regex("(?i)c", priority = 2)]
    CKw,
    #[token("char", ignore(case))]
    CharKw,
    #[token("character", ignore(case))]
    CharacterKw,
    #[token("charsetform", ignore(case))]
    CharsetformKw,
    #[token("charsetid", ignore(case))]
    CharsetidKw,
    #[token("check", ignore(case))]
    CheckKw,
    #[token("clob", ignore(case))]
    ClobKw,
    #[token("clone", ignore(case))]
    CloneKw,
    #[token("collation", ignore(case))]
    CollationKw,
    #[token("collect", ignore(case))]
    CollectKw,
    #[token("comment", ignore(case))]
    CommentKw,
    #[token("connect", ignore(case))]
    ConnectKw,
    #[token("connect_by_root", ignore(case))]
    ConnectByRootKw,
    #[token("constant", ignore(case))]
    ConstantKw,
    #[token("constraint", ignore(case))]
    ConstraintKw,
    #[token("container", ignore(case))]
    ContainerKw,
    #[token("container_map", ignore(case))]
    ContainerMapKw,
    #[token("containers_default", ignore(case))]
    ContainersDefaultKw,
    #[token("context", ignore(case))]
    ContextKw,
    #[token("create", ignore(case))]
    CreateKw,
    #[token("crossedition", ignore(case))]
    CrosseditionKw,
    #[token("cube", ignore(case))]
    CubeKw,
    #[token("current_user", ignore(case))]
    CurrentUserKw,
    #[token("cursor", ignore(case))]
    CursorKw,
    #[token("data", ignore(case))]
    DataKw,
    #[token("database", ignore(case))]
    DatabaseKw,
    #[token("date", ignore(case))]
    DateKw,
    #[token("day", ignore(case))]
    DayKw,
    #[token("db_role_change", ignore(case))]
    DbRoleChangeKw,
    #[token("ddl", ignore(case))]
    DdlKw,
    #[token("dec", ignore(case))]
    DecKw,
    #[token("decimal", ignore(case))]
    DecimalKw,
    #[token("declare", ignore(case))]
    DeclareKw,
    #[token("default", ignore(case))]
    DefaultKw,
    #[token("deferrable", ignore(case))]
    DeferrableKw,
    #[token("deferred", ignore(case))]
    DeferredKw,
    #[token("definer", ignore(case))]
    DefinerKw,
    #[token("delete", ignore(case))]
    DeleteKw,
    #[token("desc", ignore(case))]
    DescKw,
    #[token("deterministic", ignore(case))]
    DeterministicKw,
    #[token("disable", ignore(case))]
    DisableKw,
    #[token("disallow", ignore(case))]
    DisallowKw,
    #[token("disassociate", ignore(case))]
    DisassociateKw,
    #[token("double", ignore(case))]
    DoubleKw,
    #[token("drop", ignore(case))]
    DropKw,
    #[token("duration", ignore(case))]
    DurationKw,
    #[token("each", ignore(case))]
    EachKw,
    #[token("editionable", ignore(case))]
    EditionableKw,
    #[token("editioning", ignore(case))]
    EditioningKw,
    #[token("element", ignore(case))]
    ElementKw,
    #[token("else", ignore(case))]
    ElseKw,
    #[token("elsif", ignore(case))]
    ElsifKw,
    #[token("enable", ignore(case))]
    EnableKw,
    #[token("end", ignore(case))]
    EndKw,
    #[token("env", ignore(case))]
    EnvKw,
    #[token("exception", ignore(case))]
    ExceptionKw,
    #[token("exceptions", ignore(case))]
    ExceptionsKw,
    #[token("execute", ignore(case))]
    ExecuteKw,
    #[token("exists", ignore(case))]
    ExistsKw,
    #[token("extended", ignore(case))]
    ExtendedKw,
    #[token("external", ignore(case))]
    ExternalKw,
    #[token("first", ignore(case))]
    FirstKw,
    #[token("float", ignore(case))]
    FloatKw,
    #[token("follows", ignore(case))]
    FollowsKw,
    #[token("for", ignore(case))]
    ForKw,
    #[token("force", ignore(case))]
    ForceKw,
    #[token("foreign", ignore(case))]
    ForeignKw,
    #[token("forward", ignore(case))]
    ForwardKw,
    #[token("from", ignore(case))]
    FromKw,
    #[token("function", ignore(case))]
    FunctionKw,
    #[token("grant", ignore(case))]
    GrantKw,
    #[token("group", ignore(case))]
    GroupKw,
    #[token("grouping", ignore(case))]
    GroupingKw,
    #[token("having", ignore(case))]
    HavingKw,
    #[token("id", ignore(case))]
    IdKw,
    #[token("identifier", ignore(case))]
    IdentifierKw,
    #[token("if", ignore(case))]
    IfKw,
    #[token("ilike", ignore(case))]
    IlikeKw,
    #[token("immediate", ignore(case))]
    ImmediateKw,
    #[token("in", ignore(case))]
    InKw,
    #[token("index", ignore(case))]
    IndexKw,
    #[token("indicator", ignore(case))]
    IndicatorKw,
    #[token("initially", ignore(case))]
    InitiallyKw,
    #[token("insert", ignore(case))]
    InsertKw,
    #[token("instead", ignore(case))]
    InsteadKw,
    #[token("int", ignore(case))]
    IntKw,
    #[token("integer", ignore(case))]
    IntegerKw,
    #[token("interval", ignore(case))]
    IntervalKw,
    #[token("into", ignore(case))]
    IntoKw,
    #[token("invisible", ignore(case))]
    InvisibleKw,
    #[token("is", ignore(case))]
    IsKw,
    #[token("java", ignore(case))]
    JavaKw,
    #[token("key", ignore(case))]
    KeyKw,
    #[token("language", ignore(case))]
    LanguageKw,
    #[token("large", ignore(case))]
    LargeKw,
    #[token("last", ignore(case))]
    LastKw,
    #[token("length", ignore(case))]
    LengthKw,
    #[token("library", ignore(case))]
    LibraryKw,
    #[token("like", ignore(case))]
    LikeKw,
    #[token("lobs", ignore(case))]
    LobsKw,
    #[token("local", ignore(case))]
    LocalKw,
    #[token("logoff", ignore(case))]
    LogoffKw,
    #[token("logon", ignore(case))]
    LogonKw,
    #[token("long", ignore(case))]
    LongKw,
    #[token("maxlen", ignore(case))]
    MaxlenKw,
    #[token("metadata", ignore(case))]
    MetadataKw,
    #[token("mle", ignore(case))]
    MleKw,
    #[token("module", ignore(case))]
    ModuleKw,
    #[token("month", ignore(case))]
    MonthKw,
    #[token("name", ignore(case))]
    NameKw,
    #[token("national", ignore(case))]
    NationalKw,
    #[token("nchar", ignore(case))]
    NcharKw,
    #[token("nclob", ignore(case))]
    NclobKw,
    #[token("new", ignore(case))]
    NewKw,
    #[token("no", ignore(case))]
    NoKw,
    #[token("noaudit", ignore(case))]
    NoauditKw,
    #[token("nocopy", ignore(case))]
    NocopyKw,
    #[token("nocycle", ignore(case))]
    NocycleKw,
    #[token("none", ignore(case))]
    NoneKw,
    #[token("noneditionable", ignore(case))]
    NoneditionableKw,
    #[token("nonschema", ignore(case))]
    NonschemaKw,
    #[token("noprecheck", ignore(case))]
    NoprecheckKw,
    #[token("norely", ignore(case))]
    NorelyKw,
    #[token("not", ignore(case))]
    NotKw,
    #[token("novalidate", ignore(case))]
    NovalidateKw,
    #[token("null", ignore(case))]
    NullKw,
    #[token("nulls", ignore(case))]
    NullsKw,
    #[token("number", ignore(case))]
    NumberKw,
    #[token("numeric", ignore(case))]
    NumericKw,
    #[token("nvarchar2", ignore(case))]
    Nvarchar2Kw,
    #[token("object", ignore(case))]
    ObjectKw,
    #[token("of", ignore(case))]
    OfKw,
    #[token("old", ignore(case))]
    OldKw,
    #[token("on", ignore(case))]
    OnKw,
    #[token("only", ignore(case))]
    OnlyKw,
    #[token("option", ignore(case))]
    OptionKw,
    #[token("or", ignore(case))]
    OrKw,
    #[token("order", ignore(case))]
    OrderKw,
    #[token("others", ignore(case))]
    OthersKw,
    #[token("out", ignore(case))]
    OutKw,
    #[token("package", ignore(case))]
    PackageKw,
    #[token("parallel_enable", ignore(case))]
    ParallelEnableKw,
    #[token("parameters", ignore(case))]
    ParametersKw,
    #[token("parent", ignore(case))]
    ParentKw,
    #[token("pipelined", ignore(case))]
    PipelinedKw,
    #[token("plpgsql", ignore(case))]
    PlpgsqlKw,
    #[token("pls_integer", ignore(case))]
    PlsIntegerKw,
    #[token("pluggable", ignore(case))]
    PluggableKw,
    #[token("precedes", ignore(case))]
    PrecedesKw,
    #[token("precheck", ignore(case))]
    PrecheckKw,
    #[token("precision", ignore(case))]
    PrecisionKw,
    #[token("prior", ignore(case))]
    PriorKw,
    #[token("primary", ignore(case))]
    PrimaryKw,
    #[token("procedure", ignore(case))]
    ProcedureKw,
    #[token("range", ignore(case))]
    RangeKw,
    #[token("raise", ignore(case))]
    RaiseKw,
    #[token("raw", ignore(case))]
    RawKw,
    #[token("read", ignore(case))]
    ReadKw,
    #[token("real", ignore(case))]
    RealKw,
    #[token("record", ignore(case))]
    RecordKw,
    #[token("ref", ignore(case))]
    RefKw,
    #[token("reference", ignore(case))]
    ReferenceKw,
    #[token("references", ignore(case))]
    ReferencesKw,
    #[token("referencing", ignore(case))]
    ReferencingKw,
    #[token("relies_on", ignore(case))]
    ReliesOnKw,
    #[token("rely", ignore(case))]
    RelyKw,
    #[token("rename", ignore(case))]
    RenameKw,
    #[token("replace", ignore(case))]
    ReplaceKw,
    #[token("result_cache", ignore(case))]
    ResultCacheKw,
    #[token("return", ignore(case))]
    ReturnKw,
    #[token("returning", ignore(case))]
    ReturningKw,
    #[token("reverse", ignore(case))]
    ReverseKw,
    #[token("revoke", ignore(case))]
    RevokeKw,
    #[token("rollup", ignore(case))]
    RollupKw,
    #[token("row", ignore(case))]
    RowKw,
    #[token("rowid", ignore(case))]
    RowidKw,
    #[token("rowtype", ignore(case))]
    RowtypeKw,
    #[token("schema", ignore(case))]
    SchemaKw,
    #[token("scope", ignore(case))]
    ScopeKw,
    #[token("second", ignore(case))]
    SecondKw,
    #[token("select", ignore(case))]
    SelectKw,
    #[token("self", ignore(case))]
    SelfKw,
    #[token("servererror", ignore(case))]
    ServererrorKw,
    #[token("set", ignore(case))]
    SetKw,
    #[token("sets", ignore(case))]
    SetsKw,
    #[token("sharing", ignore(case))]
    SharingKw,
    #[token("shutdown", ignore(case))]
    ShutdownKw,
    #[token("siblings", ignore(case))]
    SiblingsKw,
    #[token("signature", ignore(case))]
    SignatureKw,
    #[token("smallint", ignore(case))]
    SmallintKw,
    #[token("starts", ignore(case))]
    StartsKw,
    #[token("startup", ignore(case))]
    StartupKw,
    #[token("statistics", ignore(case))]
    StatisticsKw,
    #[token("store", ignore(case))]
    StoreKw,
    #[token("string", ignore(case))]
    StringKw,
    #[token("struct", ignore(case))]
    StructKw,
    #[token("subtype", ignore(case))]
    SubtypeKw,
    #[token("suspend", ignore(case))]
    SuspendKw,
    #[token("table", ignore(case))]
    TableKw,
    #[token("tables", ignore(case))]
    TablesKw,
    #[token("tdo", ignore(case))]
    TdoKw,
    #[token("then", ignore(case))]
    ThenKw,
    #[token("time", ignore(case))]
    TimeKw,
    #[token("timestamp", ignore(case))]
    TimestampKw,
    #[token("to", ignore(case))]
    ToKw,
    #[token("trigger", ignore(case))]
    TriggerKw,
    #[token("truncate", ignore(case))]
    TruncateKw,
    #[token("type", ignore(case))]
    TypeKw,
    #[token("under", ignore(case))]
    UnderKw,
    #[token("unique", ignore(case))]
    UniqueKw,
    #[token("unplug", ignore(case))]
    UnplugKw,
    #[token("update", ignore(case))]
    UpdateKw,
    #[token("urowid", ignore(case))]
    UrowidKw,
    #[token("using", ignore(case))]
    UsingKw,
    #[token("validate", ignore(case))]
    ValidateKw,
    #[token("values", ignore(case))]
    ValuesKw,
    #[token("varchar", ignore(case))]
    VarcharKw,
    #[token("varchar2", ignore(case))]
    Varchar2Kw,
    #[token("varray", ignore(case))]
    VarrayKw,
    #[token("varrays", ignore(case))]
    VarraysKw,
    #[token("varying", ignore(case))]
    VaryingKw,
    #[token("view", ignore(case))]
    ViewKw,
    #[token("visible", ignore(case))]
    VisibleKw,
    #[token("when", ignore(case))]
    WhenKw,
    #[token("where", ignore(case))]
    WhereKw,
    #[token("with", ignore(case))]
    WithKw,
    #[token("xmlschema", ignore(case))]
    XmlschemaKw,
    #[token("xmltype", ignore(case))]
    XmltypeKw,
    #[token("year", ignore(case))]
    YearKw,
    #[token("zone", ignore(case))]
    ZoneKw,
    Error,
    #[doc = r" Marker token to indicate end of input, not used by lexer directly."]
    Eof,
}
impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Comment | Self::Whitespace)
    }
    pub fn is_punct(self) -> bool {
        matches!(
            self,
            Self::DollarQuote
                | Self::Assign
                | Self::Asterisk
                | Self::Comma
                | Self::Comparison
                | Self::Dot
                | Self::DoubleDot
                | Self::DoublePipe
                | Self::Equals
                | Self::Exclam
                | Self::LParen
                | Self::Minus
                | Self::OracleJoin
                | Self::Percentage
                | Self::Plus
                | Self::RParen
                | Self::Semicolon
                | Self::Slash
        )
    }
    pub fn is_literal(self) -> bool {
        matches!(
            self,
            Self::Integer
                | Self::Decimal
                | Self::UnquotedIdent
                | Self::QuotedIdent
                | Self::QuotedLiteral
                | Self::BindVar
        )
    }
    pub fn is_ident(self) -> bool {
        matches!(
            self,
            Self::UnquotedIdent | Self::QuotedIdent | Self::BindVar
        ) || !(self.is_trivia()
            || self.is_punct()
            || self.is_literal()
            || matches!(self, Self::Eof | Self::Error))
    }
}
impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
#[macro_export]
macro_rules ! T { [comment] => { TokenKind :: Comment } ; [whitespace] => { TokenKind :: Whitespace } ; ["$$"] => { TokenKind :: DollarQuote } ; [:=] => { TokenKind :: Assign } ; [*] => { TokenKind :: Asterisk } ; [,] => { TokenKind :: Comma } ; [comparison] => { TokenKind :: Comparison } ; [.] => { TokenKind :: Dot } ; [..] => { TokenKind :: DoubleDot } ; [||] => { TokenKind :: DoublePipe } ; [=] => { TokenKind :: Equals } ; [!] => { TokenKind :: Exclam } ; ["("] => { TokenKind :: LParen } ; [-] => { TokenKind :: Minus } ; [(+)] => { TokenKind :: OracleJoin } ; [%] => { TokenKind :: Percentage } ; [+] => { TokenKind :: Plus } ; [")"] => { TokenKind :: RParen } ; [;] => { TokenKind :: Semicolon } ; [/] => { TokenKind :: Slash } ; [int_literal] => { TokenKind :: Integer } ; [decimal_literal] => { TokenKind :: Decimal } ; [unquoted_ident] => { TokenKind :: UnquotedIdent } ; [quoted_ident] => { TokenKind :: QuotedIdent } ; [quoted_literal] => { TokenKind :: QuotedLiteral } ; [bind_var] => { TokenKind :: BindVar } ; [add] => { TokenKind :: AddKw } ; [after] => { TokenKind :: AfterKw } ; [agent] => { TokenKind :: AgentKw } ; [all] => { TokenKind :: AllKw } ; [allow] => { TokenKind :: AllowKw } ; [alter] => { TokenKind :: AlterKw } ; [analyze] => { TokenKind :: AnalyzeKw } ; [and] => { TokenKind :: AndKw } ; [annotations] => { TokenKind :: AnnotationsKw } ; [anyschema] => { TokenKind :: AnyschemaKw } ; [array] => { TokenKind :: ArrayKw } ; [as] => { TokenKind :: AsKw } ; [asc] => { TokenKind :: AscKw } ; [associate] => { TokenKind :: AssociateKw } ; [audit] => { TokenKind :: AuditKw } ; [before] => { TokenKind :: BeforeKw } ; [begin] => { TokenKind :: BeginKw } ; [bequeath] => { TokenKind :: BequeathKw } ; [between] => { TokenKind :: BetweenKw } ; [bfile] => { TokenKind :: BfileKw } ; [binary] => { TokenKind :: BinaryKw } ; [binary_double] => { TokenKind :: BinaryDoubleKw } ; [binary_float] => { TokenKind :: BinaryFloatKw } ; [binary_integer] => { TokenKind :: BinaryIntegerKw } ; [blob] => { TokenKind :: BlobKw } ; [body] => { TokenKind :: BodyKw } ; [bulk] => { TokenKind :: BulkKw } ; [by] => { TokenKind :: ByKw } ; [byte] => { TokenKind :: ByteKw } ; [call] => { TokenKind :: CallKw } ; [cascade] => { TokenKind :: CascadeKw } ; [c] => { TokenKind :: CKw } ; [char] => { TokenKind :: CharKw } ; [character] => { TokenKind :: CharacterKw } ; [charsetform] => { TokenKind :: CharsetformKw } ; [charsetid] => { TokenKind :: CharsetidKw } ; [check] => { TokenKind :: CheckKw } ; [clob] => { TokenKind :: ClobKw } ; [clone] => { TokenKind :: CloneKw } ; [collation] => { TokenKind :: CollationKw } ; [collect] => { TokenKind :: CollectKw } ; [comment] => { TokenKind :: CommentKw } ; [connect] => { TokenKind :: ConnectKw } ; [connect_by_root] => { TokenKind :: ConnectByRootKw } ; [constant] => { TokenKind :: ConstantKw } ; [constraint] => { TokenKind :: ConstraintKw } ; [container] => { TokenKind :: ContainerKw } ; [container_map] => { TokenKind :: ContainerMapKw } ; [containers_default] => { TokenKind :: ContainersDefaultKw } ; [context] => { TokenKind :: ContextKw } ; [create] => { TokenKind :: CreateKw } ; [crossedition] => { TokenKind :: CrosseditionKw } ; [cube] => { TokenKind :: CubeKw } ; [current_user] => { TokenKind :: CurrentUserKw } ; [cursor] => { TokenKind :: CursorKw } ; [data] => { TokenKind :: DataKw } ; [database] => { TokenKind :: DatabaseKw } ; [date] => { TokenKind :: DateKw } ; [day] => { TokenKind :: DayKw } ; [db_role_change] => { TokenKind :: DbRoleChangeKw } ; [ddl] => { TokenKind :: DdlKw } ; [dec] => { TokenKind :: DecKw } ; [decimal] => { TokenKind :: DecimalKw } ; [declare] => { TokenKind :: DeclareKw } ; [default] => { TokenKind :: DefaultKw } ; [deferrable] => { TokenKind :: DeferrableKw } ; [deferred] => { TokenKind :: DeferredKw } ; [definer] => { TokenKind :: DefinerKw } ; [delete] => { TokenKind :: DeleteKw } ; [desc] => { TokenKind :: DescKw } ; [deterministic] => { TokenKind :: DeterministicKw } ; [disable] => { TokenKind :: DisableKw } ; [disallow] => { TokenKind :: DisallowKw } ; [disassociate] => { TokenKind :: DisassociateKw } ; [double] => { TokenKind :: DoubleKw } ; [drop] => { TokenKind :: DropKw } ; [duration] => { TokenKind :: DurationKw } ; [each] => { TokenKind :: EachKw } ; [editionable] => { TokenKind :: EditionableKw } ; [editioning] => { TokenKind :: EditioningKw } ; [element] => { TokenKind :: ElementKw } ; [else] => { TokenKind :: ElseKw } ; [elsif] => { TokenKind :: ElsifKw } ; [enable] => { TokenKind :: EnableKw } ; [end] => { TokenKind :: EndKw } ; [env] => { TokenKind :: EnvKw } ; [exception] => { TokenKind :: ExceptionKw } ; [exceptions] => { TokenKind :: ExceptionsKw } ; [execute] => { TokenKind :: ExecuteKw } ; [exists] => { TokenKind :: ExistsKw } ; [extended] => { TokenKind :: ExtendedKw } ; [external] => { TokenKind :: ExternalKw } ; [first] => { TokenKind :: FirstKw } ; [float] => { TokenKind :: FloatKw } ; [follows] => { TokenKind :: FollowsKw } ; [for] => { TokenKind :: ForKw } ; [force] => { TokenKind :: ForceKw } ; [foreign] => { TokenKind :: ForeignKw } ; [forward] => { TokenKind :: ForwardKw } ; [from] => { TokenKind :: FromKw } ; [function] => { TokenKind :: FunctionKw } ; [grant] => { TokenKind :: GrantKw } ; [group] => { TokenKind :: GroupKw } ; [grouping] => { TokenKind :: GroupingKw } ; [having] => { TokenKind :: HavingKw } ; [id] => { TokenKind :: IdKw } ; [identifier] => { TokenKind :: IdentifierKw } ; [if] => { TokenKind :: IfKw } ; [ilike] => { TokenKind :: IlikeKw } ; [immediate] => { TokenKind :: ImmediateKw } ; [in] => { TokenKind :: InKw } ; [index] => { TokenKind :: IndexKw } ; [indicator] => { TokenKind :: IndicatorKw } ; [initially] => { TokenKind :: InitiallyKw } ; [insert] => { TokenKind :: InsertKw } ; [instead] => { TokenKind :: InsteadKw } ; [int] => { TokenKind :: IntKw } ; [integer] => { TokenKind :: IntegerKw } ; [interval] => { TokenKind :: IntervalKw } ; [into] => { TokenKind :: IntoKw } ; [invisible] => { TokenKind :: InvisibleKw } ; [is] => { TokenKind :: IsKw } ; [java] => { TokenKind :: JavaKw } ; [key] => { TokenKind :: KeyKw } ; [language] => { TokenKind :: LanguageKw } ; [large] => { TokenKind :: LargeKw } ; [last] => { TokenKind :: LastKw } ; [length] => { TokenKind :: LengthKw } ; [library] => { TokenKind :: LibraryKw } ; [like] => { TokenKind :: LikeKw } ; [lobs] => { TokenKind :: LobsKw } ; [local] => { TokenKind :: LocalKw } ; [logoff] => { TokenKind :: LogoffKw } ; [logon] => { TokenKind :: LogonKw } ; [long] => { TokenKind :: LongKw } ; [maxlen] => { TokenKind :: MaxlenKw } ; [metadata] => { TokenKind :: MetadataKw } ; [mle] => { TokenKind :: MleKw } ; [module] => { TokenKind :: ModuleKw } ; [month] => { TokenKind :: MonthKw } ; [name] => { TokenKind :: NameKw } ; [national] => { TokenKind :: NationalKw } ; [nchar] => { TokenKind :: NcharKw } ; [nclob] => { TokenKind :: NclobKw } ; [new] => { TokenKind :: NewKw } ; [no] => { TokenKind :: NoKw } ; [noaudit] => { TokenKind :: NoauditKw } ; [nocopy] => { TokenKind :: NocopyKw } ; [nocycle] => { TokenKind :: NocycleKw } ; [none] => { TokenKind :: NoneKw } ; [noneditionable] => { TokenKind :: NoneditionableKw } ; [nonschema] => { TokenKind :: NonschemaKw } ; [noprecheck] => { TokenKind :: NoprecheckKw } ; [norely] => { TokenKind :: NorelyKw } ; [not] => { TokenKind :: NotKw } ; [novalidate] => { TokenKind :: NovalidateKw } ; [null] => { TokenKind :: NullKw } ; [nulls] => { TokenKind :: NullsKw } ; [number] => { TokenKind :: NumberKw } ; [numeric] => { TokenKind :: NumericKw } ; [nvarchar2] => { TokenKind :: Nvarchar2Kw } ; [object] => { TokenKind :: ObjectKw } ; [of] => { TokenKind :: OfKw } ; [old] => { TokenKind :: OldKw } ; [on] => { TokenKind :: OnKw } ; [only] => { TokenKind :: OnlyKw } ; [option] => { TokenKind :: OptionKw } ; [or] => { TokenKind :: OrKw } ; [order] => { TokenKind :: OrderKw } ; [others] => { TokenKind :: OthersKw } ; [out] => { TokenKind :: OutKw } ; [package] => { TokenKind :: PackageKw } ; [parallel_enable] => { TokenKind :: ParallelEnableKw } ; [parameters] => { TokenKind :: ParametersKw } ; [parent] => { TokenKind :: ParentKw } ; [pipelined] => { TokenKind :: PipelinedKw } ; [plpgsql] => { TokenKind :: PlpgsqlKw } ; [pls_integer] => { TokenKind :: PlsIntegerKw } ; [pluggable] => { TokenKind :: PluggableKw } ; [precedes] => { TokenKind :: PrecedesKw } ; [precheck] => { TokenKind :: PrecheckKw } ; [precision] => { TokenKind :: PrecisionKw } ; [prior] => { TokenKind :: PriorKw } ; [primary] => { TokenKind :: PrimaryKw } ; [procedure] => { TokenKind :: ProcedureKw } ; [range] => { TokenKind :: RangeKw } ; [raise] => { TokenKind :: RaiseKw } ; [raw] => { TokenKind :: RawKw } ; [read] => { TokenKind :: ReadKw } ; [real] => { TokenKind :: RealKw } ; [record] => { TokenKind :: RecordKw } ; [ref] => { TokenKind :: RefKw } ; [reference] => { TokenKind :: ReferenceKw } ; [references] => { TokenKind :: ReferencesKw } ; [referencing] => { TokenKind :: ReferencingKw } ; [relies_on] => { TokenKind :: ReliesOnKw } ; [rely] => { TokenKind :: RelyKw } ; [rename] => { TokenKind :: RenameKw } ; [replace] => { TokenKind :: ReplaceKw } ; [result_cache] => { TokenKind :: ResultCacheKw } ; [return] => { TokenKind :: ReturnKw } ; [returning] => { TokenKind :: ReturningKw } ; [reverse] => { TokenKind :: ReverseKw } ; [revoke] => { TokenKind :: RevokeKw } ; [rollup] => { TokenKind :: RollupKw } ; [row] => { TokenKind :: RowKw } ; [rowid] => { TokenKind :: RowidKw } ; [rowtype] => { TokenKind :: RowtypeKw } ; [schema] => { TokenKind :: SchemaKw } ; [scope] => { TokenKind :: ScopeKw } ; [second] => { TokenKind :: SecondKw } ; [select] => { TokenKind :: SelectKw } ; [self] => { TokenKind :: SelfKw } ; [servererror] => { TokenKind :: ServererrorKw } ; [set] => { TokenKind :: SetKw } ; [sets] => { TokenKind :: SetsKw } ; [sharing] => { TokenKind :: SharingKw } ; [shutdown] => { TokenKind :: ShutdownKw } ; [siblings] => { TokenKind :: SiblingsKw } ; [signature] => { TokenKind :: SignatureKw } ; [smallint] => { TokenKind :: SmallintKw } ; [starts] => { TokenKind :: StartsKw } ; [startup] => { TokenKind :: StartupKw } ; [statistics] => { TokenKind :: StatisticsKw } ; [store] => { TokenKind :: StoreKw } ; [string] => { TokenKind :: StringKw } ; [struct] => { TokenKind :: StructKw } ; [subtype] => { TokenKind :: SubtypeKw } ; [suspend] => { TokenKind :: SuspendKw } ; [table] => { TokenKind :: TableKw } ; [tables] => { TokenKind :: TablesKw } ; [tdo] => { TokenKind :: TdoKw } ; [then] => { TokenKind :: ThenKw } ; [time] => { TokenKind :: TimeKw } ; [timestamp] => { TokenKind :: TimestampKw } ; [to] => { TokenKind :: ToKw } ; [trigger] => { TokenKind :: TriggerKw } ; [truncate] => { TokenKind :: TruncateKw } ; [type] => { TokenKind :: TypeKw } ; [under] => { TokenKind :: UnderKw } ; [unique] => { TokenKind :: UniqueKw } ; [unplug] => { TokenKind :: UnplugKw } ; [update] => { TokenKind :: UpdateKw } ; [urowid] => { TokenKind :: UrowidKw } ; [using] => { TokenKind :: UsingKw } ; [validate] => { TokenKind :: ValidateKw } ; [values] => { TokenKind :: ValuesKw } ; [varchar] => { TokenKind :: VarcharKw } ; [varchar2] => { TokenKind :: Varchar2Kw } ; [varray] => { TokenKind :: VarrayKw } ; [varrays] => { TokenKind :: VarraysKw } ; [varying] => { TokenKind :: VaryingKw } ; [view] => { TokenKind :: ViewKw } ; [visible] => { TokenKind :: VisibleKw } ; [when] => { TokenKind :: WhenKw } ; [where] => { TokenKind :: WhereKw } ; [with] => { TokenKind :: WithKw } ; [xmlschema] => { TokenKind :: XmlschemaKw } ; [xmltype] => { TokenKind :: XmltypeKw } ; [year] => { TokenKind :: YearKw } ; [zone] => { TokenKind :: ZoneKw } ; [EOF] => { TokenKind :: Eof } ; }
