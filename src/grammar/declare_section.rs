// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of PL/SQL declare sections from a token tree.
//! Refer to https://docs.oracle.com/en/database/oracle/oracle-database/23/lnpls/block.html#GUID-9ACEB9ED-567E-4E1A-A16A-B8B35214FC9D

use rowan::Checkpoint;

use crate::grammar::{
    opt_function_invocation, parse_datatype, parse_expr, parse_function, parse_ident,
    parse_procedure, parse_query,
};
use crate::lexer::{TokenKind, T};
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseErrorType;

pub(super) fn parse_declare_section(p: &mut Parser, checkpoint: Option<Checkpoint>) {
    let checkpoint = if let Some(checkpoint) = checkpoint {
        checkpoint
    } else {
        p.eat_ws();
        p.checkpoint()
    };
    p.start_node_at(checkpoint, SyntaxKind::DeclareSection);

    loop {
        match p.current() {
            T![cursor] => parse_cursor(p),
            T![function] => parse_function(p, true),
            T![procedure] => parse_procedure(p, true),
            T![type] => parse_type_definition(p),
            T![subtype] => parse_subtype_definition(p),
            _ => parse_item_declaration(p),
        }

        match p.current() {
            // while the docs don't specify it anywhere, `BEGIN` and `END` may not be used as an identifier here
            T![begin] | T![end] => break,
            T![cursor] | T![function] | T![procedure] | T![type] | T![subtype] => continue,
            token if token.is_ident() => continue,
            _ => break,
        }
    }

    p.finish();
}

fn parse_cursor(p: &mut Parser) {
    p.expect(T![cursor]);
    parse_ident(p, 1..1);

    if p.eat(T!["("]) {
        loop {
            parse_ident(p, 1..1);
            p.eat(T![in]);
            parse_datatype(p);

            if p.eat_one_of(&[T![:=], T![default]]) {
                parse_expr(p);
            }

            if !p.eat(T![,]) {
                break;
            }
        }

        p.expect(T![")"]);
    }

    if p.eat(T![return]) {
        parse_rowtype(p);
    }

    if p.eat(T![is]) {
        parse_query(p, false);
    } else {
        // the [`parse_query`] function already expects a trailing semi-colon
        p.expect(T![;]);
    }
}

fn parse_rowtype(p: &mut Parser) {
    parse_ident(p, 1..3);
    if p.eat(T![%]) {
        p.expect_one_of(&[T![type], T![rowtype]]);
    }
}

fn parse_type_definition(p: &mut Parser) {
    p.expect(T!(type));
    parse_ident(p, 1..1);
    p.expect(T![is]);

    match p.current() {
        // collection type
        T![table] => parse_assoc_array_type_def(p),
        T![varray] | T![varying] | T![array] => parse_varray_type_def(p),
        // record type
        T![record] => parse_record_type_definition(p),
        // ref cursor
        T![ref] => parse_ref_cursor_type_definition(p),
        // subtype
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![array],
            T![record],
            T![ref],
            T![table],
            T![varray],
            T![varying],
        ])),
    }

    p.expect(T![;]);
}

/// Combines well with nested_table_type_def
fn parse_assoc_array_type_def(p: &mut Parser) {
    p.expect(T![table]);
    p.expect(T![of]);
    parse_ident(p, 1..1);

    if p.eat(T![not]) {
        p.expect(T![null]);
    }

    if p.eat(T![index]) {
        p.expect(T![by]);
        match p.current() {
            T![binary_integer] | T![long] | T![pls_integer] => p.bump_any(),
            T![string] | T![varchar] | T![varchar2] => {
                p.bump_any();
                p.expect(T!["("]);
                p.expect(T![int_literal]);
                p.expect(T!["("]);
            }
            _ => parse_rowtype(p),
        }
    }
}

fn parse_varray_type_def(p: &mut Parser) {
    match p.current() {
        T![array] | T![varray] => p.bump_any(),
        T![varying] => {
            p.bump_any();
            p.expect(T![array]);
        }
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![array],
            T![varray],
            T![varying],
        ])),
    }

    p.expect(T!["("]);
    p.expect(T![int_literal]);
    p.expect(T![")"]);

    p.expect(T![of]);
    parse_datatype(p);

    if p.eat(T![not]) {
        p.expect(T![null]);
    }
}

fn parse_record_type_definition(p: &mut Parser) {
    p.expect(T![record]);

    p.expect(T!["("]);
    loop {
        parse_ident(p, 1..1);
        parse_datatype(p);

        if p.eat(T![not]) {
            p.expect(T![null]);
        }

        if p.eat_one_of(&[T![:=], T![default]]) {
            parse_expr(p);
        }

        if !p.eat(T![,]) {
            break;
        }
    }
    p.expect(T![")"]);
}

fn parse_ref_cursor_type_definition(p: &mut Parser) {
    p.expect(T![ref]);
    p.expect(T![cursor]);

    if p.eat(T![return]) {
        parse_rowtype(p);
    }
}

fn parse_subtype_definition(p: &mut Parser) {
    p.expect(T![subtype]);
    parse_ident(p, 1..1);
    p.expect(T![is]);

    // base type
    parse_ident(p, 1..1);
    match p.current() {
        T!["("] => {
            p.bump_any();
            p.expect(T![int_literal]);
            if p.eat(T![,]) {
                p.expect(T![int_literal]);
            }
            p.expect(T![")"]);
        }
        T![character] => {
            p.bump_any();
            p.expect(T![set]);
            parse_ident(p, 1..1);
        }
        T![range] => {
            p.bump_any();
            p.expect(T![int_literal]);
            p.expect(T![..]);
            p.expect(T![int_literal]);
        }
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T!["("],
            T![character],
            T![range],
        ])),
    }

    if p.eat(T![not]) {
        p.expect(T![null]);
    }

    p.expect(T![is]);
}

fn parse_item_declaration(p: &mut Parser) {
    parse_ident(p, 1..1);

    match p.current() {
        T![constant] => {
            p.bump_any();
            parse_datatype(p);

            if p.eat(T![not]) {
                p.expect(T![null]);
            }

            p.expect_one_of(&[T![:=], T![default]]);

            parse_expr(p);
        }
        T![exception] => p.bump_any(),
        _ => {
            parse_datatype(p);

            if p.eat(T![:=]) && !opt_function_invocation(p) {
                parse_expr(p);
            }
        }
    }

    p.expect(T![;]);
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_cursor() {
        const INPUT: &str = "CURSOR c (p_name varchar2, age number DEFAULT 18) IS
            SELECT first_name FROM employee;";

        check(
            parse(INPUT, parse_cursor),
            expect![[r#"
Root@0..97
  Keyword@0..6 "CURSOR"
  Whitespace@6..7 " "
  IdentGroup@7..8
    Ident@7..8 "c"
  Whitespace@8..9 " "
  LParen@9..10 "("
  IdentGroup@10..16
    Ident@10..16 "p_name"
  Whitespace@16..17 " "
  Datatype@17..25
    Keyword@17..25 "varchar2"
  Comma@25..26 ","
  Whitespace@26..27 " "
  IdentGroup@27..30
    Ident@27..30 "age"
  Whitespace@30..31 " "
  Datatype@31..38
    Keyword@31..37 "number"
    Whitespace@37..38 " "
  Keyword@38..45 "DEFAULT"
  Whitespace@45..46 " "
  Integer@46..48 "18"
  RParen@48..49 ")"
  Whitespace@49..50 " "
  Keyword@50..52 "IS"
  SelectStmt@52..97
    Whitespace@52..65 "\n            "
    Keyword@65..71 "SELECT"
    Whitespace@71..72 " "
    SelectClause@72..83
      ColumnExpr@72..83
        IdentGroup@72..82
          Ident@72..82 "first_name"
        Whitespace@82..83 " "
    Keyword@83..87 "FROM"
    Whitespace@87..88 " "
    IdentGroup@88..96
      Ident@88..96 "employee"
    Semicolon@96..97 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_type_definition() {
        const INPUT: &str = "TYPE custom_type IS TABLE OF table_name INDEX BY PLS_INTEGER;";
        check(
            parse(INPUT, |p| parse_declare_section(p, None)),
            expect![[r#"
Root@0..61
  DeclareSection@0..61
    Keyword@0..4 "TYPE"
    Whitespace@4..5 " "
    IdentGroup@5..16
      Ident@5..16 "custom_type"
    Whitespace@16..17 " "
    Keyword@17..19 "IS"
    Whitespace@19..20 " "
    Keyword@20..25 "TABLE"
    Whitespace@25..26 " "
    Keyword@26..28 "OF"
    Whitespace@28..29 " "
    IdentGroup@29..39
      Ident@29..39 "table_name"
    Whitespace@39..40 " "
    Keyword@40..45 "INDEX"
    Whitespace@45..46 " "
    Keyword@46..48 "BY"
    Whitespace@48..49 " "
    Keyword@49..60 "PLS_INTEGER"
    Semicolon@60..61 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_item_declarations() {
        const INPUT: &str = r#"
            p_1 NUMBER(2,1);
            p_2 NUMBER := 42;
            p_3 VARCHAR2(20);
            p_4 custom_table%ROWTYPE;
            p_5 custom_type;"#;
        check(
            parse(INPUT, |p| parse_declare_section(p, None)),
            expect![[r#"
Root@0..156
  Whitespace@0..13 "\n            "
  DeclareSection@13..156
    IdentGroup@13..16
      Ident@13..16 "p_1"
    Whitespace@16..17 " "
    Datatype@17..28
      Keyword@17..23 "NUMBER"
      LParen@23..24 "("
      Integer@24..25 "2"
      Comma@25..26 ","
      Integer@26..27 "1"
      RParen@27..28 ")"
    Semicolon@28..29 ";"
    Whitespace@29..42 "\n            "
    IdentGroup@42..45
      Ident@42..45 "p_2"
    Whitespace@45..46 " "
    Datatype@46..53
      Keyword@46..52 "NUMBER"
      Whitespace@52..53 " "
    Assign@53..55 ":="
    Whitespace@55..56 " "
    Expression@56..58
      Integer@56..58 "42"
    Semicolon@58..59 ";"
    Whitespace@59..72 "\n            "
    IdentGroup@72..75
      Ident@72..75 "p_3"
    Whitespace@75..76 " "
    Datatype@76..88
      Keyword@76..84 "VARCHAR2"
      LParen@84..85 "("
      Integer@85..87 "20"
      RParen@87..88 ")"
    Semicolon@88..89 ";"
    Whitespace@89..102 "\n            "
    IdentGroup@102..105
      Ident@102..105 "p_4"
    Whitespace@105..106 " "
    Datatype@106..126
      IdentGroup@106..118
        Ident@106..118 "custom_table"
      TypeAttribute@118..126
        Percentage@118..119 "%"
        Keyword@119..126 "ROWTYPE"
    Semicolon@126..127 ";"
    Whitespace@127..140 "\n            "
    IdentGroup@140..143
      Ident@140..143 "p_5"
    Whitespace@143..144 " "
    Datatype@144..155
      IdentGroup@144..155
        Ident@144..155 "custom_type"
    Semicolon@155..156 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_nested_procedure() {
        const INPUT: &str = r#"
PROCEDURE increment (i IN OUT NUMBER) IS
BEGIN
    NULL;
END;"#;
        check(
            parse(INPUT, |p| parse_declare_section(p, None)),
            expect![[r#"
Root@0..62
  Whitespace@0..1 "\n"
  DeclareSection@1..62
    Procedure@1..62
      ProcedureHeader@1..39
        Keyword@1..10 "PROCEDURE"
        Whitespace@10..11 " "
        IdentGroup@11..20
          Ident@11..20 "increment"
        Whitespace@20..21 " "
        ParamList@21..38
          LParen@21..22 "("
          Param@22..37
            IdentGroup@22..23
              Ident@22..23 "i"
            Whitespace@23..24 " "
            Keyword@24..26 "IN"
            Whitespace@26..27 " "
            Keyword@27..30 "OUT"
            Whitespace@30..31 " "
            Datatype@31..37
              Keyword@31..37 "NUMBER"
          RParen@37..38 ")"
        Whitespace@38..39 " "
      Keyword@39..41 "IS"
      Whitespace@41..42 "\n"
      Block@42..62
        Keyword@42..47 "BEGIN"
        Whitespace@47..52 "\n    "
        BlockStatement@52..57
          Keyword@52..56 "NULL"
          Semicolon@56..57 ";"
        Whitespace@57..58 "\n"
        Keyword@58..61 "END"
        Semicolon@61..62 ";"
"#]],
            vec![],
        );
    }
}
