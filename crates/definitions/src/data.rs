// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Specifies the token and syntax kinds to be generated

use crate::syntax::{SyntaxNode, S};
use crate::token::{Tokens, T};

pub const TOKENS: Tokens<'_> = Tokens {
    trivia: &[
        T!("comment", "comment", "comment", "--.*"),
        T!("whitespace", "whitespace", "whitespace", "[ \t\n\r]+"),
    ],
    punctuation: &[
        T!("$$", "dollar_quote", "dollar_quote"),
        T!(":=", "assign", "assign"),
        T!("*", "asterisk", "asterisk"),
        T!(",", "comma", "comma"),
        T!("comparison", "comparison", "comparison_op", "<>|<|>|<=|>="),
        T!(".", "dot", "dot"),
        T!("..", "double_dot", "range"),
        T!("||", "double_pipe", "concat"),
        T!("=", "equals", "comparison_op"),
        T!("!", "exclam", "exclam"),
        T!("(", "l_paren", "l_paren"),
        T!("-", "minus", "arithmetic_op"),
        T!("(+)", "oracle_join"),
        T!("%", "percentage", "percentage"),
        T!("+", "plus", "arithmetic_op"),
        T!(")", "r_paren", "r_paren"),
        T!(";", "semicolon", "semicolon"),
        T!("/", "slash", "slash"),
    ],
    literals: &[
        T!("int_literal", "integer", "integer", r"-?\d+", 2),
        T!(
            "unquoted_ident",
            "unquoted_ident",
            "ident",
            r"(?i)[a-z_][a-z0-9_$#]*",
            1
        ),
        T!("quoted_ident", "quoted_ident", "ident", r#""(?:[^"]|"")+""#),
        T!(
            "quoted_literal",
            "quoted_literal",
            "quoted_literal",
            "'[^']*'"
        ),
        T!("bind_var", "bind_var", "bind_var", r"(?i):[a-z][a-z0-9_]*"),
    ],
    keywords: &[
        T!("add"),
        T!("after"),
        T!("agent"),
        T!("all"),
        T!("allow"),
        T!("alter"),
        T!("analyze"),
        T!("and"),
        T!("annotations"),
        T!("anyschema"),
        T!("array"),
        T!("as"),
        T!("associate"),
        T!("audit"),
        T!("before"),
        T!("begin"),
        T!("bequeath"),
        T!("between"),
        T!("bfile"),
        T!("binary"),
        T!("binary_double"),
        T!("binary_float"),
        T!("binary_integer"),
        T!("blob"),
        T!("body"),
        T!("by"),
        T!("byte"),
        T!("call"),
        T!("cascade"),
        T!("c", "cKw", "keyword", r"(?i)c", 2), // Manual priority to not conflict with unquoted_ident
        T!("char"),
        T!("character"),
        T!("charsetform"),
        T!("charsetid"),
        T!("check"),
        T!("clob"),
        T!("clone"),
        T!("collation"),
        T!("comment"),
        T!("connect"),
        T!("connect_by_root"),
        T!("constant"),
        T!("constraint"),
        T!("container"),
        T!("container_map"),
        T!("containers_default"),
        T!("context"),
        T!("create"),
        T!("crossedition"),
        T!("current_user"),
        T!("cursor"),
        T!("data"),
        T!("database"),
        T!("date"),
        T!("day"),
        T!("db_role_change"),
        T!("ddl"),
        T!("dec"),
        T!("decimal"),
        T!("declare"),
        T!("default"),
        T!("deferrable"),
        T!("deferred"),
        T!("definer"),
        T!("delete"),
        T!("deterministic"),
        T!("disable"),
        T!("disallow"),
        T!("disassociate"),
        T!("double"),
        T!("drop"),
        T!("duration"),
        T!("each"),
        T!("editionable"),
        T!("editioning"),
        T!("element"),
        T!("else"),
        T!("elsif"),
        T!("enable"),
        T!("end"),
        T!("env"),
        T!("exception"),
        T!("exceptions"),
        T!("exists"),
        T!("extended"),
        T!("external"),
        T!("float"),
        T!("follows"),
        T!("for"),
        T!("force"),
        T!("foreign"),
        T!("forward"),
        T!("from"),
        T!("function"),
        T!("grant"),
        T!("id"),
        T!("identifier"),
        T!("if"),
        T!("ilike", "ilike", "comparison_op"),
        T!("immediate"),
        T!("in"),
        T!("index"),
        T!("indicator"),
        T!("initially"),
        T!("insert"),
        T!("instead"),
        T!("int"),
        T!("integer"),
        T!("interval"),
        T!("into"),
        T!("invisible"),
        T!("is"),
        T!("java"),
        T!("key"),
        T!("language"),
        T!("large"),
        T!("length"),
        T!("library"),
        T!("like", "like", "comparison_op"),
        T!("lobs"),
        T!("local"),
        T!("logoff"),
        T!("logon"),
        T!("long"),
        T!("maxlen"),
        T!("metadata"),
        T!("mle"),
        T!("module"),
        T!("month"),
        T!("name"),
        T!("national"),
        T!("nchar"),
        T!("nclob"),
        T!("new"),
        T!("no"),
        T!("noaudit"),
        T!("nocopy"),
        T!("nocycle"),
        T!("none"),
        T!("noneditionable"),
        T!("nonschema"),
        T!("noprecheck"),
        T!("norely"),
        T!("not"),
        T!("novalidate"),
        T!("null"),
        T!("number"),
        T!("numeric"),
        T!("nvarchar2"),
        T!("object"),
        T!("of"),
        T!("old"),
        T!("on"),
        T!("only"),
        T!("option"),
        T!("or"),
        T!("others"),
        T!("out"),
        T!("package"),
        T!("parallel_enable"),
        T!("parameters"),
        T!("parent"),
        T!("pipelined"),
        T!("plpgsql"),
        T!("pls_integer"),
        T!("pluggable"),
        T!("precedes"),
        T!("precheck"),
        T!("precision"),
        T!("prior"),
        T!("primary"),
        T!("procedure"),
        T!("range"),
        T!("raw"),
        T!("read"),
        T!("real"),
        T!("record"),
        T!("ref"),
        T!("reference"),
        T!("references"),
        T!("referencing"),
        T!("relies_on"),
        T!("rely"),
        T!("rename"),
        T!("replace"),
        T!("result_cache"),
        T!("return"),
        T!("returning"),
        T!("reverse"),
        T!("revoke"),
        T!("row"),
        T!("rowid"),
        T!("rowtype"),
        T!("schema"),
        T!("scope"),
        T!("second"),
        T!("select"),
        T!("self"),
        T!("servererror"),
        T!("set"),
        T!("sharing"),
        T!("shutdown"),
        T!("signature"),
        T!("smallint"),
        T!("starts"),
        T!("startup"),
        T!("statistics"),
        T!("store"),
        T!("string"),
        T!("struct"),
        T!("subtype"),
        T!("suspend"),
        T!("table"),
        T!("tables"),
        T!("tdo"),
        T!("then"),
        T!("time"),
        T!("timestamp"),
        T!("to"),
        T!("trigger"),
        T!("truncate"),
        T!("type"),
        T!("under"),
        T!("unique"),
        T!("unplug"),
        T!("update"),
        T!("urowid"),
        T!("using"),
        T!("validate"),
        T!("values"),
        T!("varchar"),
        T!("varchar2"),
        T!("varray"),
        T!("varrays"),
        T!("varying"),
        T!("view"),
        T!("visible"),
        T!("when"),
        T!("where"),
        T!("with"),
        T!("xmlschema"),
        T!("xmltype"),
        T!("year"),
        T!("zone"),
    ],
};

pub const SYNTAX_NODES: &'_ [SyntaxNode<'_>] = &[
    S!("alias", "An Alias for columns"),
    S!("and", "Logical operator AND"),
    S!("argument", "A singular argument inside an argument list"),
    S!("argument_list", "A list of arguments inside a `FunctionInvocation`. Made of multiple `Arguments`, separated by commas"),
    S!("arithmetic_op", "Represents an arithmetic SQL operator (+, -, *, /)"),
    S!("assign", "An Assign operator `:=`"),
    S!("asterisk", "An asterisk `*`"),
    S!("bind_var", "A bind variable, e.g. `:OLD`"),
    S!("block", "A node that marks a block"),
    S!("block_statement", "A node that marks an individual statement inside a block"),
    S!("colon", "A colon token"),
    S!("column_expr", "A single column expression, as part of an SELECT clause"),
    S!("comma", "A single comma"),
    S!("comment", "Inline comment starting with `--`"),
    S!("comparison_op", "Represents an arithmetic SQL comparison operator (=, <>, <, >, <=, >=) or other types of comparison operators of SQL (ilike, like)"),
    S!("concat", "A concatination operator `||`"),
    S!("connect_by_root", "The CONNECT_BY_ROOT operator"),
    S!("connect", "The CONNECT BY clause in selects"),
    S!("constraint", "A node that marks a full constraint"),
    S!("datatype", "Any built-in oracle datatype"),
    S!("declare_section", "A node that marks the declare section of a block"),
    S!("dollar_quote", "Single dollar quote `$$`"),
    S!("dot", "A single dot"),
    S!("error", "An error token with a cause"),
    S!("exclam", "An exclamation mark `!`"),
    S!("expression", "Holds a generic SQL logic/arithmetic expression"),
    S!("function", "A node that marks a full CREATE [..] FUNCTION block"),
    S!("function_header", "A node that marks a FUNCTION header with params and return type"),
    S!("function_invocation", "An invocation of a function, from the identifier and the opening bracket to the closing bracket"),
    S!("hierarchical_op", "An operator in hierarchical queries"),
    S!("ident", "An identifier, either quoted or unquoted"),
    S!("ident_group", "An identifier group, consisting of multiple idents"),
    S!("insert_stmt", "A node that marks a full INSERT statement"),
    S!("integer", "Any integer, positive and negative"),
    S!("into_clause", "A node that contains an `INTO` clause of a SELECT statement"),
    S!("keyword", "A SQL keyword, e.g. `CREATE`"),
    S!("logic_op", "Represents a logical SQL operator (AND, OR, NOT)"),
    S!("l_paren", "Left Paren"),
    S!("minus", "A minus `-`"),
    S!("not", "Unary logical operator NOT"),
    S!("or", "Logical operator OR"),
    S!("package", "A node that marks a full CREATE PACKAGE BODY block"),
    S!("param", "A single Param node, consisting of name & type"),
    S!("param_list", "A node that consists of multiple parameters"),
    S!("percentage", "Percentage symbol"),
    S!("plus", "A plus `+`"),
    S!("prior", "The PL/SQL unary prior operator"),
    S!("procedure", "A node that marks a full CREATE [..] PROCEDURE block"),
    S!("procedure_header", "A node that marks a PROCEDURE header with params"),
    S!("quoted_literal", "A single quoted literal"),
    S!("range", "Two dots"),
    S!("root", "The root node element"),
    S!("r_paren", "Right Paren"),
    S!("select_clause", "A node that contains the whole SELECT clause of a query"),
    S!("select_stmt", "A node that marks a full SELECT statement"),
    S!("semicolon", "A semi colon"),
    S!("slash", "Slash char `/`"),
    S!("starts", "A STARTS WITH clause in a SELECT statement"),
    S!("text", "A text slice node"),
    S!("trigger","A node that marks a full CREATE [..] TRIGGER block"),
    S!("trigger_header","A node that marks a TRIGGER header"),
    S!("type_attribute", "A `%TYPE` attribute"),
    S!("type_name", "A type name"),
    S!("variable_decl", "A node that marks a variable declaration as part of a function or procedure"),
    S!("variable_decl_list", "A node that marks a list of variable declarations of functions and procedures"),
    S!("view", "A node that marks a full CREATE VIEW block"),
    S!("where_clause", "Represent a complete `WHERE` clause expression"),
    S!("whitespace", "Any whitespace character"),
];
