// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of tables from a token tree.

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

use super::*;

/// Parses a complete table.
pub(crate) fn parse_table(p: &mut Parser) {
    p.start(SyntaxKind::Table);

    parse_header(p);
    parse_body(p);

    while !p.at(TokenKind::Eof) {
        p.bump_any();
    }
    p.finish();
}

fn parse_header(p: &mut Parser) {
    p.expect(TokenKind::CreateKw);

    match p.current() {
        TokenKind::GlobalKw | TokenKind::PrivateKw => {
            p.bump_any();
            p.expect(TokenKind::TemporaryKw);
        }
        TokenKind::ImmutableKw => {
            p.bump_any();
            p.eat(TokenKind::BlockchainKw);
        }
        TokenKind::ShardedKw | TokenKind::DuplicatedKw | TokenKind::BlockchainKw => {
            p.bump_any();
        }
        _ => {}
    }

    p.expect(TokenKind::TableKw);

    parse_qualified_ident(p, 1..2);
}

fn parse_body(p: &mut Parser) {
    p.expect(TokenKind::LParen);

    p.start(SyntaxKind::ColumnList);
    parse_column(p);
    while p.eat(TokenKind::Comma) {
        parse_column(p);
    }
    p.finish();

    p.expect(TokenKind::RParen);
}

fn parse_column(p: &mut Parser) {
    p.start(SyntaxKind::Column);

    parse_qualified_ident(p, 1..1);

    parse_datatype(p);

    // TODO: sort

    // TODO: visible / invisible

    if p.at(TokenKind::DefaultKw) {
        p.start(SyntaxKind::DefaultExpression);
        p.bump_any();
        if p.eat(TokenKind::OnKw) {
            p.expect(TokenKind::NullKw);
        }
        parse_expr(p);
        p.finish();
    }

    // TODO: identity clause

    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_table_with_one_simple_column() {
        check(
            parse("CREATE TABLE hello (id number)", parse_table),
            expect![[r#"
Root@0..30
  Table@0..30
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..12 "TABLE"
    Whitespace@12..13 " "
    Ident@13..18 "hello"
    Whitespace@18..19 " "
    LParen@19..20 "("
    ColumnList@20..29
      Column@20..29
        Ident@20..22 "id"
        Datatype@22..29
          Whitespace@22..23 " "
          Keyword@23..29 "number"
    RParen@29..30 ")"
"#]],
        );
    }

    #[test]
    fn test_table_with_fully_qualified_identifiers() {
        check(
            parse(
                "CREATE TABLE \"MY_SCHEMA\".\"MY_TABLE\" (\"MY_COLUMN\" \"MY_SCHEMA\".\"MY_TABLE\".\"MY_COLUMN\"%type)",
                parse_table,
            ),
            expect![[r#"
Root@0..89
  Table@0..89
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..12 "TABLE"
    QualifiedIdent@12..35
      Whitespace@12..13 " "
      Ident@13..24 "\"MY_SCHEMA\""
      Dot@24..25 "."
      Ident@25..35 "\"MY_TABLE\""
    Whitespace@35..36 " "
    LParen@36..37 "("
    ColumnList@37..88
      Column@37..88
        Ident@37..48 "\"MY_COLUMN\""
        Datatype@48..88
          Whitespace@48..49 " "
          QualifiedIdent@49..83
            Ident@49..60 "\"MY_SCHEMA\""
            Dot@60..61 "."
            Ident@61..71 "\"MY_TABLE\""
            Dot@71..72 "."
            Ident@72..83 "\"MY_COLUMN\""
          TypeAttribute@83..88
            Percentage@83..84 "%"
            Keyword@84..88 "type"
    RParen@88..89 ")"
"#]],
        );
    }

    #[test]
    fn test_default_expression() {
        check(
            parse(
                r#"CREATE TABLE tab("EMPLOYEE_ID" NUMBER(6,0) DEFAULT HR.DEPT_SEQ.nextval)"#,
                parse_table,
            ),
            expect![[r#"
Root@0..71
  Table@0..71
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..12 "TABLE"
    Whitespace@12..13 " "
    Ident@13..16 "tab"
    LParen@16..17 "("
    ColumnList@17..70
      Column@17..70
        Ident@17..30 "\"EMPLOYEE_ID\""
        Datatype@30..43
          Whitespace@30..31 " "
          Keyword@31..37 "NUMBER"
          LParen@37..38 "("
          Integer@38..39 "6"
          Comma@39..40 ","
          Integer@40..41 "0"
          RParen@41..42 ")"
          Whitespace@42..43 " "
        DefaultExpression@43..70
          Keyword@43..50 "DEFAULT"
          Whitespace@50..51 " "
          QualifiedIdent@51..70
            Ident@51..53 "HR"
            Dot@53..54 "."
            Ident@54..62 "DEPT_SEQ"
            Dot@62..63 "."
            Ident@63..70 "nextval"
    RParen@70..71 ")"
"#]],
        );
    }

    #[test]
    fn test_table_with_multiple_columns() {
        check(
            parse(
                r#"CREATE TABLE "MY_SCHEMA"."EMP" (
  "EMP_ID" NUMBER(6,0) DEFAULT MY_SCEHMA.EMP_SEQ.nextval,
  "CREATED_AT" TIMESTAMP WITH TIME ZONE,
  "EMAIL" VARCHAR2(25 BYTE))"#,
                parse_table,
            ),
            expect![[r#"
Root@0..160
  Table@0..160
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..12 "TABLE"
    QualifiedIdent@12..30
      Whitespace@12..13 " "
      Ident@13..24 "\"MY_SCHEMA\""
      Dot@24..25 "."
      Ident@25..30 "\"EMP\""
    Whitespace@30..31 " "
    LParen@31..32 "("
    ColumnList@32..159
      Column@32..89
        Whitespace@32..35 "\n  "
        Ident@35..43 "\"EMP_ID\""
        Datatype@43..56
          Whitespace@43..44 " "
          Keyword@44..50 "NUMBER"
          LParen@50..51 "("
          Integer@51..52 "6"
          Comma@52..53 ","
          Integer@53..54 "0"
          RParen@54..55 ")"
          Whitespace@55..56 " "
        DefaultExpression@56..89
          Keyword@56..63 "DEFAULT"
          Whitespace@63..64 " "
          QualifiedIdent@64..89
            Ident@64..73 "MY_SCEHMA"
            Dot@73..74 "."
            Ident@74..81 "EMP_SEQ"
            Dot@81..82 "."
            Ident@82..89 "nextval"
      Comma@89..90 ","
      Column@90..130
        Whitespace@90..93 "\n  "
        Ident@93..105 "\"CREATED_AT\""
        Datatype@105..130
          Whitespace@105..106 " "
          Keyword@106..115 "TIMESTAMP"
          Whitespace@115..116 " "
          Keyword@116..120 "WITH"
          Whitespace@120..121 " "
          Keyword@121..125 "TIME"
          Whitespace@125..126 " "
          Keyword@126..130 "ZONE"
      Comma@130..131 ","
      Column@131..159
        Whitespace@131..134 "\n  "
        Ident@134..141 "\"EMAIL\""
        Datatype@141..159
          Whitespace@141..142 " "
          Keyword@142..150 "VARCHAR2"
          LParen@150..151 "("
          Integer@151..153 "25"
          Whitespace@153..154 " "
          Keyword@154..158 "BYTE"
          RParen@158..159 ")"
    RParen@159..160 ")"
"#]],
        );
    }

    mod test_table_modifiers {
        use super::*;

        #[test]
        fn test_global_temporary_table() {
            check(
                parse("CREATE GLOBAL TEMPORARY TABLE tab (id number)", parse_table),
                expect![[r#"
Root@0..45
  Table@0..45
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..13 "GLOBAL"
    Whitespace@13..14 " "
    Keyword@14..23 "TEMPORARY"
    Whitespace@23..24 " "
    Keyword@24..29 "TABLE"
    Whitespace@29..30 " "
    Ident@30..33 "tab"
    Whitespace@33..34 " "
    LParen@34..35 "("
    ColumnList@35..44
      Column@35..44
        Ident@35..37 "id"
        Datatype@37..44
          Whitespace@37..38 " "
          Keyword@38..44 "number"
    RParen@44..45 ")"
"#]],
            );
        }

        #[test]
        fn test_private_temporary_table() {
            check(
                parse(
                    "CREATE PRIVATE TEMPORARY TABLE tab (id number)",
                    parse_table,
                ),
                expect![[r#"
Root@0..46
  Table@0..46
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..14 "PRIVATE"
    Whitespace@14..15 " "
    Keyword@15..24 "TEMPORARY"
    Whitespace@24..25 " "
    Keyword@25..30 "TABLE"
    Whitespace@30..31 " "
    Ident@31..34 "tab"
    Whitespace@34..35 " "
    LParen@35..36 "("
    ColumnList@36..45
      Column@36..45
        Ident@36..38 "id"
        Datatype@38..45
          Whitespace@38..39 " "
          Keyword@39..45 "number"
    RParen@45..46 ")"
"#]],
            );
        }

        #[test]
        fn test_sharded_table() {
            check(
                parse("CREATE SHARDED TABLE tab (id number)", parse_table),
                expect![[r#"
Root@0..36
  Table@0..36
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..14 "SHARDED"
    Whitespace@14..15 " "
    Keyword@15..20 "TABLE"
    Whitespace@20..21 " "
    Ident@21..24 "tab"
    Whitespace@24..25 " "
    LParen@25..26 "("
    ColumnList@26..35
      Column@26..35
        Ident@26..28 "id"
        Datatype@28..35
          Whitespace@28..29 " "
          Keyword@29..35 "number"
    RParen@35..36 ")"
"#]],
            );
        }

        #[test]
        fn test_duplicated_table() {
            check(
                parse("CREATE DUPLICATED TABLE tab (id number)", parse_table),
                expect![[r#"
Root@0..39
  Table@0..39
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..17 "DUPLICATED"
    Whitespace@17..18 " "
    Keyword@18..23 "TABLE"
    Whitespace@23..24 " "
    Ident@24..27 "tab"
    Whitespace@27..28 " "
    LParen@28..29 "("
    ColumnList@29..38
      Column@29..38
        Ident@29..31 "id"
        Datatype@31..38
          Whitespace@31..32 " "
          Keyword@32..38 "number"
    RParen@38..39 ")"
"#]],
            );
        }

        #[test]
        fn test_immutable_blockchain_table() {
            check(
                parse(
                    "CREATE IMMUTABLE BLOCKCHAIN TABLE tab (id number)",
                    parse_table,
                ),
                expect![[r#"
Root@0..49
  Table@0..49
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..16 "IMMUTABLE"
    Whitespace@16..17 " "
    Keyword@17..27 "BLOCKCHAIN"
    Whitespace@27..28 " "
    Keyword@28..33 "TABLE"
    Whitespace@33..34 " "
    Ident@34..37 "tab"
    Whitespace@37..38 " "
    LParen@38..39 "("
    ColumnList@39..48
      Column@39..48
        Ident@39..41 "id"
        Datatype@41..48
          Whitespace@41..42 " "
          Keyword@42..48 "number"
    RParen@48..49 ")"
"#]],
            );
        }

        #[test]
        fn test_blockchain_table() {
            check(
                parse("CREATE BLOCKCHAIN TABLE tab (id number)", parse_table),
                expect![[r#"
Root@0..39
  Table@0..39
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..17 "BLOCKCHAIN"
    Whitespace@17..18 " "
    Keyword@18..23 "TABLE"
    Whitespace@23..24 " "
    Ident@24..27 "tab"
    Whitespace@27..28 " "
    LParen@28..29 "("
    ColumnList@29..38
      Column@29..38
        Ident@29..31 "id"
        Datatype@31..38
          Whitespace@31..32 " "
          Keyword@32..38 "number"
    RParen@38..39 ")"
"#]],
            );
        }

        #[test]
        fn test_immutable_table() {
            check(
                parse("CREATE IMMUTABLE TABLE tab (id number)", parse_table),
                expect![[r#"
Root@0..38
  Table@0..38
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..16 "IMMUTABLE"
    Whitespace@16..17 " "
    Keyword@17..22 "TABLE"
    Whitespace@22..23 " "
    Ident@23..26 "tab"
    Whitespace@26..27 " "
    LParen@27..28 "("
    ColumnList@28..37
      Column@28..37
        Ident@28..30 "id"
        Datatype@30..37
          Whitespace@30..31 " "
          Keyword@31..37 "number"
    RParen@37..38 ")"
"#]],
            );
        }
    }
}
