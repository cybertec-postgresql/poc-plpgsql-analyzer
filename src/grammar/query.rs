// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of procedures from a token tree.

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

use super::parse_expr;

pub(crate) fn parse_query(p: &mut Parser) {
    p.start(SyntaxKind::SelectStmt);
    p.expect(TokenKind::SelectKw);
    parse_column_expr(p);
    p.expect(TokenKind::FromKw);
    parse_from_list(p);

    if p.at(TokenKind::WhereKw) {
        parse_where_clause(p);
    }

    p.eat(TokenKind::SemiColon);
    p.finish();
}

fn parse_column_expr(p: &mut Parser) {
    if p.eat(TokenKind::Asterisk) {
        return;
    }

    p.start(SyntaxKind::SelectClause);

    while !p.at(TokenKind::FromKw) && !p.at(TokenKind::Eof) && !p.at(TokenKind::SemiColon) {
        p.start(SyntaxKind::ColumnExpr);

        parse_expr(p);

        p.finish();

        p.eat(TokenKind::Comma);
    }

    p.finish();
}

fn parse_from_list(p: &mut Parser) {
    while p.expect_one_of(&[TokenKind::UnquotedIdent, TokenKind::QuotedIdent])
        && p.eat(TokenKind::Comma)
    {}
}

fn parse_where_clause(p: &mut Parser) {
    p.start(SyntaxKind::WhereClause);
    p.expect(TokenKind::WhereKw);

    parse_expr(p);

    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_parse_simple_select() {
        check(
            parse("SELECT * FROM table", parse_query),
            expect![[r#"
Root@0..19
  SelectStmt@0..19
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 " "
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    Ident@14..19 "table"
"#]],
        );
    }

    #[test]
    fn test_parse_function_invocation() {
        check(
            parse("SELECT trunc(SYSDATE, 'MM') FROM DUAL;", parse_query),
            expect![[r#"
Root@0..38
  SelectStmt@0..38
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..28
      ColumnExpr@7..28
        FunctionInvocation@7..27
          IdentGroup@7..12
            Ident@7..12 "trunc"
          LParen@12..13 "("
          ArgumentList@13..26
            Argument@13..20
              IdentGroup@13..20
                Ident@13..20 "SYSDATE"
            Comma@20..21 ","
            Whitespace@21..22 " "
            Argument@22..26
              QuotedLiteral@22..26 "'MM'"
          RParen@26..27 ")"
        Whitespace@27..28 " "
    Keyword@28..32 "FROM"
    Whitespace@32..33 " "
    Ident@33..37 "DUAL"
    SemiColon@37..38 ";"
"#]],
        );
    }

    #[test]
    fn test_parse_oracle_left_join() {
        const INPUT: &str = include_str!("../../tests/dql/select_left_join.ora.sql");

        check(
            parse(INPUT, parse_query),
            expect![[r#"
Root@0..328
  SelectStmt@0..94
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 "\n"
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    Ident@14..21 "persons"
    Comma@21..22 ","
    Whitespace@22..23 " "
    Ident@23..29 "places"
    Whitespace@29..30 "\n"
    WhereClause@30..93
      Keyword@30..35 "WHERE"
      Expression@35..93
        Whitespace@35..38 "\n  "
        Comment@38..58 "-- LEFT (OUTER) JOIN"
        Whitespace@58..61 "\n  "
        IdentGroup@61..77
          Ident@61..67 "places"
          Dot@67..68 "."
          Ident@68..77 "person_id"
        Keyword@77..80 "(+)"
        Whitespace@80..81 " "
        ComparisonOp@81..82 "="
        Whitespace@82..83 " "
        IdentGroup@83..93
          Ident@83..90 "persons"
          Dot@90..91 "."
          Ident@91..93 "id"
    SemiColon@93..94 ";"
  Whitespace@94..97 "\n  "
  Comment@97..131 "-- Can be switched, s ..."
  Whitespace@131..134 "\n  "
  Comment@134..170 "-- persons.id = place ..."
  Whitespace@170..173 "\n  "
  Comment@173..175 "--"
  Whitespace@175..178 "\n  "
  Comment@178..248 "-- Valid syntax: whit ..."
  Whitespace@248..251 "\n  "
  Comment@251..288 "-- places.person_id ( ..."
  Whitespace@288..291 "\n  "
  Comment@291..327 "-- places.person_id ( ..."
  Whitespace@327..328 "\n"
"#]],
        );
    }

    #[test]
    fn test_parse_complex_where_clause() {
        const INPUT: &str = include_str!("../../tests/dql/multiple_where_conditions.ora.sql");

        check(
            parse(INPUT, parse_query),
            expect![[r#"
Root@0..72
  SelectStmt@0..71
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 "\n"
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    Ident@14..15 "a"
    Comma@15..16 ","
    Whitespace@16..17 " "
    Ident@17..18 "b"
    Comma@18..19 ","
    Whitespace@19..20 " "
    Ident@20..21 "c"
    Whitespace@21..22 "\n"
    WhereClause@22..70
      Keyword@22..27 "WHERE"
      Expression@27..70
        Expression@27..38
          Whitespace@27..28 " "
          Integer@28..31 "100"
          Whitespace@31..32 " "
          ComparisonOp@32..33 "<"
          Whitespace@33..34 " "
          IdentGroup@34..35
            Ident@34..35 "a"
          Whitespace@35..38 "\n  "
        LogicOp@38..41 "AND"
        Whitespace@41..42 " "
        LParen@42..43 "("
        Expression@43..68
          Expression@43..51
            IdentGroup@43..44
              Ident@43..44 "b"
            Whitespace@44..45 " "
            ComparisonOp@45..47 "<="
            Whitespace@47..48 " "
            Integer@48..50 "50"
            Whitespace@50..51 " "
          LogicOp@51..53 "OR"
          Expression@53..68
            Whitespace@53..54 " "
            IdentGroup@54..55
              Ident@54..55 "c"
            Whitespace@55..56 " "
            ComparisonOp@56..60 "LIKE"
            Whitespace@60..61 " "
            QuotedLiteral@61..68 "'%foo%'"
        RParen@68..69 ")"
        Whitespace@69..70 "\n"
    SemiColon@70..71 ";"
  Whitespace@71..72 "\n"
"#]],
        );
    }
}
