// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of procedures from a token tree.

use crate::grammar::{opt_expr, parse_expr, parse_ident};
use crate::parser::{safe_loop, Parser};
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

pub(crate) fn parse_query(p: &mut Parser, expect_into_clause: bool) {
    p.start(SyntaxKind::SelectStmt);
    p.expect(T![select]);
    parse_column_expr(p);
    parse_into_clause(p, expect_into_clause);
    p.expect(T![from]);
    parse_from_list(p);

    while p.at(T![where]) || p.at(T![connect]) || p.at(T![starts]) {
        if p.at(T![where]) {
            parse_where_clause(p);
        } else if p.at(T![connect]) {
            parse_connect_by(p);
        } else {
            parse_starts_with(p);
        }
    }

    p.eat(T![;]);
    p.finish();
}

pub(crate) fn parse_connect_by(p: &mut Parser) {
    p.start(SyntaxKind::Connect);
    p.expect(T![connect]);
    p.expect(T![by]);
    p.eat(T![nocycle]);
    parse_expr(p);
    if p.eat(T![starts]) {
        p.expect(T![with]);
        parse_expr(p);
    }
    p.finish()
}

pub(crate) fn parse_starts_with(p: &mut Parser) {
    p.start(SyntaxKind::Starts);
    p.expect(T![starts]);
    p.expect(T![with]);
    parse_expr(p);
    p.expect(T![connect]);
    p.expect(T![by]);
    p.eat(T![nocycle]);
    parse_expr(p);
    p.finish()
}

pub(crate) fn parse_insert(p: &mut Parser) {
    p.start(SyntaxKind::InsertStmt);
    p.expect(T![insert]);
    p.expect(T![into]);
    parse_ident(p, 1..2);
    parse_ident(p, 0..1);

    if p.eat(T!["("]) {
        safe_loop!(p, {
            parse_ident(p, 1..1);
            if !p.eat(T![,]) {
                break;
            }
        });
        p.expect(T![")"]);
    }

    p.expect(T![values]);
    p.expect(T!["("]);

    safe_loop!(p, {
        if !opt_expr(p) {
            p.expect(T![default]);
        }
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);

    if p.eat_one_of(&[T![return], T![returning]]) {
        safe_loop!(p, {
            parse_expr(p);
            if !p.eat(T![,]) {
                break;
            }
        });
        p.expect(T![into]);
        safe_loop!(p, {
            parse_ident(p, 1..1);
            if !p.eat(T![,]) {
                break;
            }
        });
    }

    p.eat(T![;]);
    p.finish();
}

fn parse_column_expr(p: &mut Parser) {
    if p.eat(T![*]) {
        return;
    }

    p.start(SyntaxKind::SelectClause);

    safe_loop!(p, {
        p.start(SyntaxKind::ColumnExpr);

        parse_expr(p);

        p.finish();

        p.eat(T![,]);

        if [T![into], T![from], T![EOF], T![;]].contains(&p.current()) {
            break;
        }
    });

    p.finish();
}

fn parse_into_clause(p: &mut Parser, expect_into_clause: bool) {
    let checkpoint = p.checkpoint();

    if expect_into_clause {
        if !p.expect(T![into]) {
            return;
        }
    } else if !p.eat(T![into]) {
        return;
    }

    safe_loop!(p, {
        parse_ident(p, 1..1);
        if !p.eat(T![,]) {
            break;
        }
    });

    p.start_node_at(checkpoint, SyntaxKind::IntoClause);
    p.finish();
}

fn parse_from_list(p: &mut Parser) {
    safe_loop!(p, {
        parse_ident(p, 1..1);
        if !p.eat(T![,]) {
            break;
        }
    });
}

fn parse_where_clause(p: &mut Parser) {
    p.start(SyntaxKind::WhereClause);
    p.expect(T![where]);

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
            parse("SELECT * FROM table", |p| parse_query(p, false)),
            expect![[r#"
Root@0..19
  SelectStmt@0..19
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 " "
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    IdentGroup@14..19
      Ident@14..19 "table"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_select_into_clause() {
        check(
            parse("SELECT 1 INTO x FROM table", |p| parse_query(p, false)),
            expect![[r#"
Root@0..26
  SelectStmt@0..26
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..9
      ColumnExpr@7..9
        Integer@7..8 "1"
        Whitespace@8..9 " "
    IntoClause@9..16
      Keyword@9..13 "INTO"
      Whitespace@13..14 " "
      IdentGroup@14..15
        Ident@14..15 "x"
      Whitespace@15..16 " "
    Keyword@16..20 "FROM"
    Whitespace@20..21 " "
    IdentGroup@21..26
      Ident@21..26 "table"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_function_invocation() {
        check(
            parse("SELECT trunc(SYSDATE, 'MM') FROM DUAL;", |p| {
                parse_query(p, false)
            }),
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
    IdentGroup@33..37
      Ident@33..37 "DUAL"
    Semicolon@37..38 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_oracle_left_join() {
        const INPUT: &str = include_str!("../../tests/dql/select_left_join.ora.sql");

        check(
            parse(INPUT, |p| parse_query(p, false)),
            expect![[r#"
Root@0..328
  SelectStmt@0..94
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 "\n"
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    IdentGroup@14..21
      Ident@14..21 "persons"
    Comma@21..22 ","
    Whitespace@22..23 " "
    IdentGroup@23..29
      Ident@23..29 "places"
    Whitespace@29..30 "\n"
    WhereClause@30..93
      Keyword@30..35 "WHERE"
      Whitespace@35..38 "\n  "
      Comment@38..58 "-- LEFT (OUTER) JOIN"
      Whitespace@58..61 "\n  "
      Expression@61..93
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
    Semicolon@93..94 ";"
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
            vec![],
        );
    }

    #[test]
    fn test_parse_complex_where_clause() {
        const INPUT: &str = include_str!("../../tests/dql/multiple_where_conditions.ora.sql");

        check(
            parse(INPUT, |p| parse_query(p, false)),
            expect![[r#"
Root@0..72
  SelectStmt@0..71
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 "\n"
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    IdentGroup@14..15
      Ident@14..15 "a"
    Comma@15..16 ","
    Whitespace@16..17 " "
    IdentGroup@17..18
      Ident@17..18 "b"
    Comma@18..19 ","
    Whitespace@19..20 " "
    IdentGroup@20..21
      Ident@20..21 "c"
    Whitespace@21..22 "\n"
    WhereClause@22..70
      Keyword@22..27 "WHERE"
      Whitespace@27..28 " "
      Expression@28..70
        Expression@28..38
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
          Whitespace@53..54 " "
          Expression@54..68
            IdentGroup@54..55
              Ident@54..55 "c"
            Whitespace@55..56 " "
            ComparisonOp@56..60 "LIKE"
            Whitespace@60..61 " "
            QuotedLiteral@61..68 "'%foo%'"
        RParen@68..69 ")"
        Whitespace@69..70 "\n"
    Semicolon@70..71 ";"
  Whitespace@71..72 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_insert() {
        check(
            parse(
                r#"INSERT INTO job_history j (id, d_id)
                    VALUES (p_emp_id, DEFAULT)
                    RETURNING p_emp_id + 1, 'abc' INTO id, name;"#,
                parse_insert,
            ),
            expect![[r#"
Root@0..148
  InsertStmt@0..148
    Keyword@0..6 "INSERT"
    Whitespace@6..7 " "
    Keyword@7..11 "INTO"
    Whitespace@11..12 " "
    IdentGroup@12..23
      Ident@12..23 "job_history"
    Whitespace@23..24 " "
    IdentGroup@24..25
      Ident@24..25 "j"
    Whitespace@25..26 " "
    LParen@26..27 "("
    IdentGroup@27..29
      Ident@27..29 "id"
    Comma@29..30 ","
    Whitespace@30..31 " "
    IdentGroup@31..35
      Ident@31..35 "d_id"
    RParen@35..36 ")"
    Whitespace@36..57 "\n                    "
    Keyword@57..63 "VALUES"
    Whitespace@63..64 " "
    LParen@64..65 "("
    IdentGroup@65..73
      Ident@65..73 "p_emp_id"
    Comma@73..74 ","
    Whitespace@74..75 " "
    IdentGroup@75..82
      Ident@75..82 "DEFAULT"
    RParen@82..83 ")"
    Whitespace@83..104 "\n                    "
    Keyword@104..113 "RETURNING"
    Whitespace@113..114 " "
    Expression@114..126
      IdentGroup@114..122
        Ident@114..122 "p_emp_id"
      Whitespace@122..123 " "
      ArithmeticOp@123..124 "+"
      Whitespace@124..125 " "
      Integer@125..126 "1"
    Comma@126..127 ","
    Whitespace@127..128 " "
    QuotedLiteral@128..133 "'abc'"
    Whitespace@133..134 " "
    Keyword@134..138 "INTO"
    Whitespace@138..139 " "
    IdentGroup@139..141
      Ident@139..141 "id"
    Comma@141..142 ","
    Whitespace@142..143 " "
    IdentGroup@143..147
      Ident@143..147 "name"
    Semicolon@147..148 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_connect_by() {
        check(
            parse(
                r#"SELECT employee_id, last_name, manager_id, LEVEL FROM employees CONNECT BY PRIOR employee_id = manager_id;"#,
                |p| parse_query(p, false),
            ),
            expect![[r#"
Root@0..106
  SelectStmt@0..106
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..49
      ColumnExpr@7..18
        IdentGroup@7..18
          Ident@7..18 "employee_id"
      Comma@18..19 ","
      Whitespace@19..20 " "
      ColumnExpr@20..29
        IdentGroup@20..29
          Ident@20..29 "last_name"
      Comma@29..30 ","
      Whitespace@30..31 " "
      ColumnExpr@31..41
        IdentGroup@31..41
          Ident@31..41 "manager_id"
      Comma@41..42 ","
      Whitespace@42..43 " "
      ColumnExpr@43..49
        IdentGroup@43..48
          Ident@43..48 "LEVEL"
        Whitespace@48..49 " "
    Keyword@49..53 "FROM"
    Whitespace@53..54 " "
    IdentGroup@54..63
      Ident@54..63 "employees"
    Whitespace@63..64 " "
    Connect@64..105
      Keyword@64..71 "CONNECT"
      Whitespace@71..72 " "
      Keyword@72..74 "BY"
      Whitespace@74..75 " "
      Expression@75..105
        Expression@75..93
          LogicOp@75..80 "PRIOR"
          Whitespace@80..81 " "
          IdentGroup@81..92
            Ident@81..92 "employee_id"
          Whitespace@92..93 " "
        ComparisonOp@93..94 "="
        Whitespace@94..95 " "
        IdentGroup@95..105
          Ident@95..105 "manager_id"
    Semicolon@105..106 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_connect_by_root() {
        check(
            parse(
                r#"SELECT last_name, CONNECT_BY_ROOT last_name
   FROM employees
   WHERE LEVEL > 1 and department_id = 110
   CONNECT BY PRIOR employee_id = manager_id;"#,
                |p| parse_query(p, false),
            ),
            expect![[r#"
Root@0..150
  SelectStmt@0..150
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..47
      ColumnExpr@7..16
        IdentGroup@7..16
          Ident@7..16 "last_name"
      Comma@16..17 ","
      Whitespace@17..18 " "
      ColumnExpr@18..47
        Expression@18..47
          LogicOp@18..33 "CONNECT_BY_ROOT"
          Whitespace@33..34 " "
          IdentGroup@34..43
            Ident@34..43 "last_name"
          Whitespace@43..47 "\n   "
    Keyword@47..51 "FROM"
    Whitespace@51..52 " "
    IdentGroup@52..61
      Ident@52..61 "employees"
    Whitespace@61..65 "\n   "
    WhereClause@65..108
      Keyword@65..70 "WHERE"
      Whitespace@70..71 " "
      Expression@71..108
        Expression@71..81
          IdentGroup@71..76
            Ident@71..76 "LEVEL"
          Whitespace@76..77 " "
          ComparisonOp@77..78 ">"
          Whitespace@78..79 " "
          Integer@79..80 "1"
          Whitespace@80..81 " "
        LogicOp@81..84 "and"
        Whitespace@84..85 " "
        Expression@85..108
          IdentGroup@85..98
            Ident@85..98 "department_id"
          Whitespace@98..99 " "
          ComparisonOp@99..100 "="
          Whitespace@100..101 " "
          Integer@101..104 "110"
          Whitespace@104..108 "\n   "
    Connect@108..149
      Keyword@108..115 "CONNECT"
      Whitespace@115..116 " "
      Keyword@116..118 "BY"
      Whitespace@118..119 " "
      Expression@119..149
        Expression@119..137
          LogicOp@119..124 "PRIOR"
          Whitespace@124..125 " "
          IdentGroup@125..136
            Ident@125..136 "employee_id"
          Whitespace@136..137 " "
        ComparisonOp@137..138 "="
        Whitespace@138..139 " "
        IdentGroup@139..149
          Ident@139..149 "manager_id"
    Semicolon@149..150 ";"
"#]],
            vec![],
        );
    }
}
