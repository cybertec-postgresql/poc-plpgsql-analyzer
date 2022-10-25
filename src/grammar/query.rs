// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of procedures from a token tree.

use super::parse_ident;
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

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

    p.start(SyntaxKind::ColumnExprList);

    while !p.at(TokenKind::FromKw) {
        p.start(SyntaxKind::ColumnExpr);

        if !p.eat(TokenKind::Ident) {
            p.until_last(TokenKind::Comma);
        }

        p.finish();
    }

    p.finish();
}

fn parse_from_list(p: &mut Parser) {
    loop {
        p.expect(TokenKind::Ident);

        if !p.eat(TokenKind::Comma) {
            break;
        }
    }
}

fn parse_where_clause(p: &mut Parser) {
    p.start(SyntaxKind::WhereClauseList);
    p.expect(TokenKind::WhereKw);

    loop {
        parse_where_expr(p);

        if !p.eat(TokenKind::Comma) {
            break;
        }
    }

    p.finish();
}

fn parse_where_expr(p: &mut Parser) {
    p.start(SyntaxKind::WhereClause);
    parse_ident(p);

    p.eat(TokenKind::OracleJoin);
    p.expect(TokenKind::Equals);
    parse_ident(p);

    p.finish();
}

#[cfg(test)]
mod tests {
    use super::super::tests::{check, parse};
    use super::*;
    use expect_test::expect;

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
    WhereClauseList@30..93
      Keyword@30..35 "WHERE"
      WhereClause@35..93
        Whitespace@35..38 "\n  "
        Comment@38..58 "-- LEFT (OUTER) JOIN"
        Whitespace@58..61 "\n  "
        Ident@61..77 "places.person_id"
        Keyword@77..80 "(+)"
        Whitespace@80..81 " "
        Equals@81..82 "="
        Whitespace@82..83 " "
        Ident@83..93 "persons.id"
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
}
