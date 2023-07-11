// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! A lax implementation for parsing datatypes from a token tree.
//! See https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/Data-Types.html#GUID-A3C0D836-BADB-44E5-A5D4-265BA5968483

use crate::grammar::{parse_expr, parse_ident};
use crate::lexer::{TokenKind, T};
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseError;

/// Parses a complete datatype.
pub fn parse_datatype(p: &mut Parser) {
    p.start(SyntaxKind::Datatype);

    let datatype = p.current();

    // All datatypes except intervals may be handled the same
    match datatype {
        T![interval] => {
            p.bump_any();
            p.expect_one_of(&[T![year], T![day]]);

            if p.eat(T!["("]) {
                parse_expr(p);
                p.expect(T![")"]);
            }

            p.expect(T![to]);
            p.expect_one_of(&[T![month], T![second]]);

            if p.eat(T!["("]) {
                parse_expr(p);
                p.expect(T![")"]);
            }
        }
        T![char]
        | T![varchar2]
        | T![nchar]
        | T![nvarchar2]
        | T![number]
        | T![float]
        | T![binary_float]
        | T![binary_double]
        | T![long]
        | T![raw]
        | T![date]
        | T![timestamp]
        | T![blob]
        | T![clob]
        | T![nclob]
        | T![bfile]
        | T![rowid]
        | T![urowid]
        | T![character]
        | T![varchar]
        | T![national]
        | T![numeric]
        | T![decimal]
        | T![dec]
        | T![integer]
        | T![int]
        | T![smallint]
        | T![double]
        | T![real]
        | T![string]
        | T![binary_integer]
        | T![pls_integer] => {
            p.bump_any();

            match datatype {
                T![character] | T![char] | T![nchar] => {
                    p.eat(T![varying]);
                }
                T![national] => {
                    p.expect_one_of(&[T![character], T![char]]);
                    p.eat(T![varying]);
                }
                T![double] => {
                    p.eat(T![precision]);
                }
                T![long] => {
                    p.eat(T![raw]);
                }
                _ => {}
            }

            if p.eat(T!["("]) {
                p.expect_one_of(&[T![int_literal], T![*]]);

                if p.eat(T![,]) {
                    p.expect(T![int_literal]);
                }

                p.eat_one_of(&[T![char], T![byte]]);

                p.expect(T![")"]);
            }

            match p.current() {
                T![with] => {
                    p.bump_any();
                    p.eat(T![local]);
                    p.expect(T![time]);
                    p.expect(T![zone]);
                }
                T![character] => {
                    p.bump_any();
                    p.expect(T![set]);
                    parse_ident(p, 1..3);
                }
                _ => {}
            }
        }
        T![unquoted_ident] | T![quoted_ident] => {
            parse_ident(p, 1..3);
            let checkpoint = p.checkpoint();
            if p.eat(T![%]) {
                p.expect_one_of(&[T![type], T![rowtype]]);
                p.start_node_at(checkpoint, SyntaxKind::TypeAttribute);
                p.finish();
            }
        }
        _ => p.error(ParseError::UnknownToken(datatype.to_string())),
    }

    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_varchar2() {
        check(
            parse("varchar2(2)", parse_datatype),
            expect![[r#"
Root@0..11
  Datatype@0..11
    Keyword@0..8 "varchar2"
    LParen@8..9 "("
    Integer@9..10 "2"
    RParen@10..11 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_varchar2_with_byte() {
        check(
            parse("varchar2(2 byte)", parse_datatype),
            expect![[r#"
Root@0..16
  Datatype@0..16
    Keyword@0..8 "varchar2"
    LParen@8..9 "("
    Integer@9..10 "2"
    Whitespace@10..11 " "
    Keyword@11..15 "byte"
    RParen@15..16 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_varchar2_with_char() {
        check(
            parse("varchar2(2 char)", parse_datatype),
            expect![[r#"
Root@0..16
  Datatype@0..16
    Keyword@0..8 "varchar2"
    LParen@8..9 "("
    Integer@9..10 "2"
    Whitespace@10..11 " "
    Keyword@11..15 "char"
    RParen@15..16 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_number() {
        check(
            parse("number", parse_datatype),
            expect![[r#"
Root@0..6
  Datatype@0..6
    Keyword@0..6 "number"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_number_with_precision() {
        check(
            parse("number(1)", parse_datatype),
            expect![[r#"
Root@0..9
  Datatype@0..9
    Keyword@0..6 "number"
    LParen@6..7 "("
    Integer@7..8 "1"
    RParen@8..9 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_number_with_precision_and_scale() {
        check(
            parse("number(1, 2)", parse_datatype),
            expect![[r#"
Root@0..12
  Datatype@0..12
    Keyword@0..6 "number"
    LParen@6..7 "("
    Integer@7..8 "1"
    Comma@8..9 ","
    Whitespace@9..10 " "
    Integer@10..11 "2"
    RParen@11..12 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_long_raw() {
        check(
            parse("long raw", parse_datatype),
            expect![[r#"
Root@0..8
  Datatype@0..8
    Keyword@0..4 "long"
    Whitespace@4..5 " "
    Keyword@5..8 "raw"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_timestamp() {
        check(
            parse("timestamp(6) with local time zone", parse_datatype),
            expect![[r#"
Root@0..33
  Datatype@0..33
    Keyword@0..9 "timestamp"
    LParen@9..10 "("
    Integer@10..11 "6"
    RParen@11..12 ")"
    Whitespace@12..13 " "
    Keyword@13..17 "with"
    Whitespace@17..18 " "
    Keyword@18..23 "local"
    Whitespace@23..24 " "
    Keyword@24..28 "time"
    Whitespace@28..29 " "
    Keyword@29..33 "zone"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_interval() {
        check(
            parse("interval day(1) to second (2)", parse_datatype),
            expect![[r#"
Root@0..29
  Datatype@0..29
    Keyword@0..8 "interval"
    Whitespace@8..9 " "
    Keyword@9..12 "day"
    LParen@12..13 "("
    Integer@13..14 "1"
    RParen@14..15 ")"
    Whitespace@15..16 " "
    Keyword@16..18 "to"
    Whitespace@18..19 " "
    Keyword@19..25 "second"
    Whitespace@25..26 " "
    LParen@26..27 "("
    Integer@27..28 "2"
    RParen@28..29 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_char_varying() {
        check(
            parse("char varying (2)", parse_datatype),
            expect![[r#"
Root@0..16
  Datatype@0..16
    Keyword@0..4 "char"
    Whitespace@4..5 " "
    Keyword@5..12 "varying"
    Whitespace@12..13 " "
    LParen@13..14 "("
    Integer@14..15 "2"
    RParen@15..16 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_national_char_varying() {
        check(
            parse("national char varying (20)", parse_datatype),
            expect![[r#"
Root@0..26
  Datatype@0..26
    Keyword@0..8 "national"
    Whitespace@8..9 " "
    Keyword@9..13 "char"
    Whitespace@13..14 " "
    Keyword@14..21 "varying"
    Whitespace@21..22 " "
    LParen@22..23 "("
    Integer@23..25 "20"
    RParen@25..26 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_any_type() {
        check(
            parse("sys.anytype", parse_datatype),
            expect![[r#"
Root@0..11
  Datatype@0..11
    IdentGroup@0..11
      Ident@0..3 "sys"
      Dot@3..4 "."
      Ident@4..11 "anytype"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_user_defined_type() {
        check(
            parse("my_schema.my_type", parse_datatype),
            expect![[r#"
Root@0..17
  Datatype@0..17
    IdentGroup@0..17
      Ident@0..9 "my_schema"
      Dot@9..10 "."
      Ident@10..17 "my_type"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_table_column_type() {
        check(
            parse("my_schema.my_table.my_column%type", parse_datatype),
            expect![[r#"
Root@0..33
  Datatype@0..33
    IdentGroup@0..28
      Ident@0..9 "my_schema"
      Dot@9..10 "."
      Ident@10..18 "my_table"
      Dot@18..19 "."
      Ident@19..28 "my_column"
    TypeAttribute@28..33
      Percentage@28..29 "%"
      Keyword@29..33 "type"
"#]],
            vec![],
        );
    }
}
