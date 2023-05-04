// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! A lax implementation for parsing datatypes from a token tree.
//! See https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/Data-Types.html#GUID-A3C0D836-BADB-44E5-A5D4-265BA5968483

use crate::grammar::{parse_expr, parse_ident};
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseError;

/// Parses a complete datatype.
pub fn parse_datatype(p: &mut Parser) {
    p.start(SyntaxKind::Datatype);

    let datatype = p.current();

    // All datatypes except intervals may be handled the same
    match datatype {
        TokenKind::IntervalKw => {
            p.bump_any();
            p.expect_one_of(&[TokenKind::YearKw, TokenKind::DayKw]);

            if p.eat(TokenKind::LParen) {
                parse_expr(p);
                p.expect(TokenKind::RParen);
            }

            p.expect(TokenKind::ToKw);
            p.expect_one_of(&[TokenKind::MonthKw, TokenKind::SecondKw]);

            if p.eat(TokenKind::LParen) {
                parse_expr(p);
                p.expect(TokenKind::RParen);
            }
        }
        TokenKind::CharKw
        | TokenKind::Varchar2Kw
        | TokenKind::NcharKw
        | TokenKind::Nvarchar2Kw
        | TokenKind::NumberKw
        | TokenKind::FloatKw
        | TokenKind::BinaryFloatKw
        | TokenKind::BinaryDoubleKw
        | TokenKind::LongKw
        | TokenKind::RawKw
        | TokenKind::DateKw
        | TokenKind::TimestampKw
        | TokenKind::BlobKw
        | TokenKind::ClobKw
        | TokenKind::NclobKw
        | TokenKind::BfileKw
        | TokenKind::RowidKw
        | TokenKind::UrowidKw
        | TokenKind::CharacterKw
        | TokenKind::VarcharKw
        | TokenKind::NationalKw
        | TokenKind::NumericKw
        | TokenKind::DecimalKw
        | TokenKind::DecKw
        | TokenKind::IntegerKw
        | TokenKind::IntKw
        | TokenKind::SmallintKw
        | TokenKind::DoubleKw
        | TokenKind::RealKw => {
            p.bump_any();

            match datatype {
                TokenKind::CharacterKw | TokenKind::CharKw | TokenKind::NcharKw => {
                    p.eat(TokenKind::VaryingKw);
                }
                TokenKind::NationalKw => {
                    p.expect_one_of(&[TokenKind::CharacterKw, TokenKind::CharKw]);
                    p.eat(TokenKind::VaryingKw);
                }
                TokenKind::DoubleKw => {
                    p.eat(TokenKind::PrecisionKw);
                }
                TokenKind::LongKw => {
                    p.eat(TokenKind::RawKw);
                }
                _ => {}
            }

            if p.eat(TokenKind::LParen) {
                p.expect_one_of(&[TokenKind::Integer, TokenKind::Asterisk]);

                if p.eat(TokenKind::Comma) {
                    p.expect(TokenKind::Integer);
                }

                p.eat_one_of(&[TokenKind::CharKw, TokenKind::ByteKw]);

                p.expect(TokenKind::RParen);
            }

            match p.current() {
                TokenKind::WithKw => {
                    p.bump_any();
                    p.eat(TokenKind::LocalKw);
                    p.expect(TokenKind::TimeKw);
                    p.expect(TokenKind::ZoneKw);
                }
                TokenKind::CharacterKw => {
                    p.bump_any();
                    p.expect(TokenKind::SetKw);
                    parse_ident(p, 1..3);
                }
                _ => {}
            }
        }
        TokenKind::UnquotedIdent | TokenKind::QuotedIdent => {
            parse_ident(p, 1..3);
            let checkpoint = p.checkpoint();
            if p.eat(TokenKind::Percentage) {
                p.expect(TokenKind::TypeKw);
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
        );
    }
}
