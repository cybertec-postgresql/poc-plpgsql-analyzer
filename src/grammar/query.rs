// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of procedures from a token tree.

use crate::grammar::{opt_expr, parse_expr, parse_function, parse_ident, parse_procedure};
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

    if p.at(T![where]) {
        parse_where_clause(p);
    }

    match p.current() {
        T![connect] => parse_connect_by(p),
        T![starts] => parse_starts_with(p),
        _ => (), // No-op
    }

    if p.at(T![order]) {
        parse_order_by_clause(p);
    }

    p.eat(T![;]);
    p.finish();
}

pub(crate) fn parse_cte(p: &mut Parser) {
    p.start(SyntaxKind::WithClause);
    p.expect(T![with]);
    if [T![function], T![procedure]].contains(&p.current()) {
        parse_plsql_declarations(p);
    }
    safe_loop!(p, {
        if p.nth(1) == Some(T![analytic]) {
            parse_subav_factoring_clause(p);
        } else {
            if !p.at(T![select]) {
                parse_subquery_factoring_clause(p);
            }
        }

        if !p.eat(T![,]) || p.at(T![select]) {
            break;
        }
    });
    p.finish();
    parse_query(p, false);
}

pub(crate) fn parse_subav_factoring_clause(p: &mut Parser) {
    p.start(SyntaxKind::SubavFactoringClause);
    parse_ident(p, 1..1);
    p.expect(T![analytic]);
    p.expect(T![view]);
    p.expect(T![as]);
    p.expect(T!["("]);
    parse_subav_clause(p);
    p.expect(T![")"]);
    p.finish();
}

pub(crate) fn parse_subav_clause(p: &mut Parser) {
    p.start(SyntaxKind::SubavClause);
    p.expect(T![using]);
    parse_ident(p, 1..2);
    if p.at(T![hierarchies]) {
        parse_hierarchies_clause(p);
    }
    if p.at(T![filter]) {
        parse_filter_clauses(p);
    }
    if p.at(T![add]) {
        parse_add_calcs_clause(p);
    }
    p.finish();
}

pub(crate) fn parse_hierarchies_clause(p: &mut Parser) {
    p.start(SyntaxKind::HierarchiesClause);
    p.expect(T![hierarchies]);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_ident(p, 1..2);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.finish();
}

pub(crate) fn parse_filter_clauses(p: &mut Parser) {
    p.start(SyntaxKind::FilterClauses);
    p.expect(T![filter]);
    p.expect(T![fact]);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_filter_clause(p);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.finish();
}

pub(crate) fn parse_filter_clause(p: &mut Parser) {
    p.start(SyntaxKind::FilterClause);
    if !p.eat(T![measures]) {
        parse_ident(p, 1..2);
    }
    p.expect(T![to]);
    parse_expr(p);
    p.finish();
}

pub(crate) fn parse_add_calcs_clause(p: &mut Parser) {
    p.start(SyntaxKind::AddCalcsClause);
    p.expect(T![add]);
    p.expect(T![measures]);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_calc_meas_clause(p);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.finish();
}

pub(crate) fn parse_calc_meas_clause(p: &mut Parser) {
    p.start(SyntaxKind::CalcMeasClause);
    parse_ident(p, 1..1);
    p.expect(T![as]);
    p.expect(T!["("]);
    parse_expr(p);
    p.expect(T![")"]);
    p.finish();
}

pub(crate) fn parse_subquery_factoring_clause(p: &mut Parser) {
    p.start(SyntaxKind::SubqueryFactoringClause);
    parse_ident(p, 1..2);
    if p.eat(T!["("]) {
        safe_loop!(p, {
            parse_ident(p, 1..1);
            if !p.eat(T![,]) {
                break;
            }
        });
        p.eat(T![")"]);
    }
    p.expect(T![as]);
    p.expect(T!["("]);
    parse_query(p, false);
    p.expect(T![")"]);
    if p.at(T![search]) {
        parse_search_clause(p);
    }
    if p.at(T![cycle]) {
        parse_cycle_clause(p);
    }
    p.finish();
}

pub(crate) fn parse_plsql_declarations(p: &mut Parser) {
    safe_loop!(p, {
        match p.current() {
            T![function] => parse_function(p, true),
            T![procedure] => parse_procedure(p, true),
            _ => break,
        }
    });
}

pub(crate) fn parse_search_clause(p: &mut Parser) {
    p.start(SyntaxKind::SearchClause);
    p.expect(T![search]);
    p.expect_one_of(&[T![breadth], T![depth]]);
    p.expect(T![first]);
    p.expect(T![by]);
    safe_loop!(p, {
        parse_ident(p, 1..1);
        p.eat_one_of(&[T![asc], T![desc]]);
        if p.eat(T![nulls]) {
            p.expect_one_of(&[T![first], T![last]]);
        }

        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![set]);
    parse_ident(p, 1..2);
    p.finish();
}

pub(crate) fn parse_cycle_clause(p: &mut Parser) {
    p.start(SyntaxKind::CycleClause);
    p.expect(T![cycle]);
    safe_loop!(p, {
        parse_ident(p, 1..1);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![set]);
    parse_ident(p, 1..1);
    p.expect(T![set]);
    parse_expr(p);
    p.expect(T![default]);
    parse_expr(p);
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
        if [T![as], T![quoted_ident], T![unquoted_ident]].contains(&p.current()) {
            parse_alias(p);
        }

        p.finish();

        p.eat(T![,]);

        if [T![into], T![from], T![EOF], T![;]].contains(&p.current()) {
            break;
        }
    });

    p.finish();
}

fn parse_alias(p: &mut Parser) {
    p.start(SyntaxKind::Alias);
    p.eat(T![as]);
    p.expect_one_of(&[T![quoted_ident], T![unquoted_ident]]);
    p.finish()
}

pub(crate) fn parse_into_clause(p: &mut Parser, expect_into_clause: bool) {
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

pub(crate) fn parse_where_clause(p: &mut Parser) {
    p.start(SyntaxKind::WhereClause);
    p.expect(T![where]);

    parse_expr(p);

    p.finish();
}

pub(crate) fn parse_order_by_clause(p: &mut Parser) {
    p.start(SyntaxKind::OrderByClause);
    p.expect(T![order]);
    p.eat(T![siblings]);
    p.expect(T![by]);
    safe_loop!(p, {
        if !p.eat(T![int_literal]) {
            parse_expr(p);
        }
        p.eat_one_of(&[T![asc], T![desc]]);

        if p.eat(T![nulls]) {
            p.expect_one_of(&[T![first], T![last]]);
        }
        if !p.eat(T![,]) {
            break;
        }
    });
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
    fn test_parse_select_with_alias() {
        check(
            parse(r#"SELECT name "Name" FROM table"#, |p| {
                parse_query(p, false)
            }),
            expect![[r#"
Root@0..29
  SelectStmt@0..29
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..19
      ColumnExpr@7..19
        IdentGroup@7..11
          Ident@7..11 "name"
        Whitespace@11..12 " "
        Alias@12..18
          Ident@12..18 "\"Name\""
        Whitespace@18..19 " "
    Keyword@19..23 "FROM"
    Whitespace@23..24 " "
    IdentGroup@24..29
      Ident@24..29 "table"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_select_with_as_alias() {
        check(
            parse(r#"SELECT name as "Name" FROM table"#, |p| {
                parse_query(p, false)
            }),
            expect![[r#"
Root@0..32
  SelectStmt@0..32
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..22
      ColumnExpr@7..22
        IdentGroup@7..11
          Ident@7..11 "name"
        Whitespace@11..12 " "
        Alias@12..21
          Keyword@12..14 "as"
          Whitespace@14..15 " "
          Ident@15..21 "\"Name\""
        Whitespace@21..22 " "
    Keyword@22..26 "FROM"
    Whitespace@26..27 " "
    IdentGroup@27..32
      Ident@27..32 "table"
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
              Expression@13..20
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
    Expression@65..73
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
        Expression@7..18
          IdentGroup@7..18
            Ident@7..18 "employee_id"
      Comma@18..19 ","
      Whitespace@19..20 " "
      ColumnExpr@20..29
        Expression@20..29
          IdentGroup@20..29
            Ident@20..29 "last_name"
      Comma@29..30 ","
      Whitespace@30..31 " "
      ColumnExpr@31..41
        Expression@31..41
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
          HierarchicalOp@75..80 "PRIOR"
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
                r#"SELECT last_name "Employee", CONNECT_BY_ROOT last_name "Manager",
   LEVEL-1, SYS_CONNECT_BY_PATH(last_name, '/') 
   FROM employees
   WHERE LEVEL > 1 and department_id = 110
   CONNECT BY PRIOR employee_id = manager_id;"#,
                |p| parse_query(p, false),
            ),
            expect![[r#"
Root@0..221
  SelectStmt@0..221
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    SelectClause@7..118
      ColumnExpr@7..27
        IdentGroup@7..16
          Ident@7..16 "last_name"
        Whitespace@16..17 " "
        Alias@17..27
          Ident@17..27 "\"Employee\""
      Comma@27..28 ","
      Whitespace@28..29 " "
      ColumnExpr@29..64
        Expression@29..55
          HierarchicalOp@29..44 "CONNECT_BY_ROOT"
          Whitespace@44..45 " "
          IdentGroup@45..54
            Ident@45..54 "last_name"
          Whitespace@54..55 " "
        Alias@55..64
          Ident@55..64 "\"Manager\""
      Comma@64..65 ","
      Whitespace@65..69 "\n   "
      ColumnExpr@69..74
        IdentGroup@69..74
          Ident@69..74 "LEVEL"
      ColumnExpr@74..76
        Expression@74..76
          Integer@74..76 "-1"
      Comma@76..77 ","
      Whitespace@77..78 " "
      ColumnExpr@78..118
        FunctionInvocation@78..113
          IdentGroup@78..97
            Ident@78..97 "SYS_CONNECT_BY_PATH"
          LParen@97..98 "("
          ArgumentList@98..112
            Argument@98..107
              Expression@98..107
                IdentGroup@98..107
                  Ident@98..107 "last_name"
            Comma@107..108 ","
            Whitespace@108..109 " "
            Argument@109..112
              QuotedLiteral@109..112 "'/'"
          RParen@112..113 ")"
        Whitespace@113..118 " \n   "
    Keyword@118..122 "FROM"
    Whitespace@122..123 " "
    IdentGroup@123..132
      Ident@123..132 "employees"
    Whitespace@132..136 "\n   "
    WhereClause@136..179
      Keyword@136..141 "WHERE"
      Whitespace@141..142 " "
      Expression@142..179
        Expression@142..152
          IdentGroup@142..147
            Ident@142..147 "LEVEL"
          Whitespace@147..148 " "
          ComparisonOp@148..149 ">"
          Whitespace@149..150 " "
          Integer@150..151 "1"
          Whitespace@151..152 " "
        LogicOp@152..155 "and"
        Whitespace@155..156 " "
        Expression@156..179
          IdentGroup@156..169
            Ident@156..169 "department_id"
          Whitespace@169..170 " "
          ComparisonOp@170..171 "="
          Whitespace@171..172 " "
          Integer@172..175 "110"
          Whitespace@175..179 "\n   "
    Connect@179..220
      Keyword@179..186 "CONNECT"
      Whitespace@186..187 " "
      Keyword@187..189 "BY"
      Whitespace@189..190 " "
      Expression@190..220
        Expression@190..208
          HierarchicalOp@190..195 "PRIOR"
          Whitespace@195..196 " "
          IdentGroup@196..207
            Ident@196..207 "employee_id"
          Whitespace@207..208 " "
        ComparisonOp@208..209 "="
        Whitespace@209..210 " "
        IdentGroup@210..220
          Ident@210..220 "manager_id"
    Semicolon@220..221 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_query_with_order_by() {
        check(
            parse("SELECT * FROM emp ORDER BY salary ASC;", |p| {
                parse_query(p, false)
            }),
            expect![[r#"
Root@0..38
  SelectStmt@0..38
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 " "
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    IdentGroup@14..17
      Ident@14..17 "emp"
    Whitespace@17..18 " "
    OrderByClause@18..37
      Keyword@18..23 "ORDER"
      Whitespace@23..24 " "
      Keyword@24..26 "BY"
      Whitespace@26..27 " "
      IdentGroup@27..33
        Ident@27..33 "salary"
      Whitespace@33..34 " "
      Keyword@34..37 "ASC"
    Semicolon@37..38 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_query_order_by_multiple() {
        check(
            parse("SELECT * FROM emp ORDER BY salary, name DESC;", |p| {
                parse_query(p, false)
            }),
            expect![[r#"
Root@0..45
  SelectStmt@0..45
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 " "
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    IdentGroup@14..17
      Ident@14..17 "emp"
    Whitespace@17..18 " "
    OrderByClause@18..44
      Keyword@18..23 "ORDER"
      Whitespace@23..24 " "
      Keyword@24..26 "BY"
      Whitespace@26..27 " "
      Expression@27..33
        IdentGroup@27..33
          Ident@27..33 "salary"
      Comma@33..34 ","
      Whitespace@34..35 " "
      IdentGroup@35..39
        Ident@35..39 "name"
      Whitespace@39..40 " "
      Keyword@40..44 "DESC"
    Semicolon@44..45 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_query_order_by_nulls_first() {
        check(
            parse("SELECT * FROM emp ORDER BY salary NULLS FIRST;", |p| {
                parse_query(p, false)
            }),
            expect![[r#"
Root@0..46
  SelectStmt@0..46
    Keyword@0..6 "SELECT"
    Whitespace@6..7 " "
    Asterisk@7..8 "*"
    Whitespace@8..9 " "
    Keyword@9..13 "FROM"
    Whitespace@13..14 " "
    IdentGroup@14..17
      Ident@14..17 "emp"
    Whitespace@17..18 " "
    OrderByClause@18..45
      Keyword@18..23 "ORDER"
      Whitespace@23..24 " "
      Keyword@24..26 "BY"
      Whitespace@26..27 " "
      IdentGroup@27..33
        Ident@27..33 "salary"
      Whitespace@33..34 " "
      Keyword@34..39 "NULLS"
      Whitespace@39..40 " "
      Keyword@40..45 "FIRST"
    Semicolon@45..46 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_cte() {
        check(parse("WITH CTE AS (SELECT name, employee_id FROM employee WHERE city = 'Delhi') select * from CTE;", parse_cte),
            expect![[r#"
Root@0..92
  WithClause@0..74
    Keyword@0..4 "WITH"
    Whitespace@4..5 " "
    SubqueryFactoringClause@5..74
      IdentGroup@5..8
        Ident@5..8 "CTE"
      Whitespace@8..9 " "
      Keyword@9..11 "AS"
      Whitespace@11..12 " "
      LParen@12..13 "("
      SelectStmt@13..72
        Keyword@13..19 "SELECT"
        Whitespace@19..20 " "
        SelectClause@20..38
          ColumnExpr@20..24
            Expression@20..24
              IdentGroup@20..24
                Ident@20..24 "name"
          Comma@24..25 ","
          Whitespace@25..26 " "
          ColumnExpr@26..38
            IdentGroup@26..37
              Ident@26..37 "employee_id"
            Whitespace@37..38 " "
        Keyword@38..42 "FROM"
        Whitespace@42..43 " "
        IdentGroup@43..51
          Ident@43..51 "employee"
        Whitespace@51..52 " "
        WhereClause@52..72
          Keyword@52..57 "WHERE"
          Whitespace@57..58 " "
          Expression@58..72
            IdentGroup@58..62
              Ident@58..62 "city"
            Whitespace@62..63 " "
            ComparisonOp@63..64 "="
            Whitespace@64..65 " "
            QuotedLiteral@65..72 "'Delhi'"
      RParen@72..73 ")"
      Whitespace@73..74 " "
  SelectStmt@74..92
    Keyword@74..80 "select"
    Whitespace@80..81 " "
    Asterisk@81..82 "*"
    Whitespace@82..83 " "
    Keyword@83..87 "from"
    Whitespace@87..88 " "
    IdentGroup@88..91
      Ident@88..91 "CTE"
    Semicolon@91..92 ";"
"#]],
            vec![]);
    }

    #[test]
    fn test_multi_cte() {
        check(
            parse(
                "WITH CTE AS (SELECT name, employee_id FROM employee),
CTE1 AS (SELECT employee_id, vehicle_name FROM vehicle)
SELECT name, vehicle_name FROM CTE;",
                parse_cte,
            ),
            expect![[r#"
Root@0..145
  WithClause@0..110
    Keyword@0..4 "WITH"
    Whitespace@4..5 " "
    SubqueryFactoringClause@5..52
      IdentGroup@5..8
        Ident@5..8 "CTE"
      Whitespace@8..9 " "
      Keyword@9..11 "AS"
      Whitespace@11..12 " "
      LParen@12..13 "("
      SelectStmt@13..51
        Keyword@13..19 "SELECT"
        Whitespace@19..20 " "
        SelectClause@20..38
          ColumnExpr@20..24
            Expression@20..24
              IdentGroup@20..24
                Ident@20..24 "name"
          Comma@24..25 ","
          Whitespace@25..26 " "
          ColumnExpr@26..38
            IdentGroup@26..37
              Ident@26..37 "employee_id"
            Whitespace@37..38 " "
        Keyword@38..42 "FROM"
        Whitespace@42..43 " "
        IdentGroup@43..51
          Ident@43..51 "employee"
      RParen@51..52 ")"
    Comma@52..53 ","
    Whitespace@53..54 "\n"
    SubqueryFactoringClause@54..110
      IdentGroup@54..58
        Ident@54..58 "CTE1"
      Whitespace@58..59 " "
      Keyword@59..61 "AS"
      Whitespace@61..62 " "
      LParen@62..63 "("
      SelectStmt@63..108
        Keyword@63..69 "SELECT"
        Whitespace@69..70 " "
        SelectClause@70..96
          ColumnExpr@70..81
            Expression@70..81
              IdentGroup@70..81
                Ident@70..81 "employee_id"
          Comma@81..82 ","
          Whitespace@82..83 " "
          ColumnExpr@83..96
            IdentGroup@83..95
              Ident@83..95 "vehicle_name"
            Whitespace@95..96 " "
        Keyword@96..100 "FROM"
        Whitespace@100..101 " "
        IdentGroup@101..108
          Ident@101..108 "vehicle"
      RParen@108..109 ")"
      Whitespace@109..110 "\n"
  SelectStmt@110..145
    Keyword@110..116 "SELECT"
    Whitespace@116..117 " "
    SelectClause@117..136
      ColumnExpr@117..121
        Expression@117..121
          IdentGroup@117..121
            Ident@117..121 "name"
      Comma@121..122 ","
      Whitespace@122..123 " "
      ColumnExpr@123..136
        IdentGroup@123..135
          Ident@123..135 "vehicle_name"
        Whitespace@135..136 " "
    Keyword@136..140 "FROM"
    Whitespace@140..141 " "
    IdentGroup@141..144
      Ident@141..144 "CTE"
    Semicolon@144..145 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_cte_function() {
        check(
            parse(
                "WITH FUNCTION text_length(a CLOB) 
   RETURN NUMBER DETERMINISTIC IS
BEGIN 
  RETURN DBMS_LOB.GETLENGTH(a);
END;
SELECT text_length('hans') FROM DUAL;",
                parse_cte,
            ),
            expect![[r#"
Root@0..150
  WithClause@0..113
    Keyword@0..4 "WITH"
    Whitespace@4..5 " "
    Function@5..113
      FunctionHeader@5..66
        Keyword@5..13 "FUNCTION"
        Whitespace@13..14 " "
        IdentGroup@14..25
          Ident@14..25 "text_length"
        ParamList@25..33
          LParen@25..26 "("
          Param@26..32
            IdentGroup@26..27
              Ident@26..27 "a"
            Whitespace@27..28 " "
            Datatype@28..32
              Keyword@28..32 "CLOB"
          RParen@32..33 ")"
        Whitespace@33..38 " \n   "
        Keyword@38..44 "RETURN"
        Whitespace@44..45 " "
        Datatype@45..52
          Keyword@45..51 "NUMBER"
          Whitespace@51..52 " "
        Keyword@52..65 "DETERMINISTIC"
        Whitespace@65..66 " "
      Keyword@66..68 "IS"
      Whitespace@68..69 "\n"
      Block@69..112
        Keyword@69..74 "BEGIN"
        Whitespace@74..78 " \n  "
        BlockStatement@78..107
          Keyword@78..84 "RETURN"
          Whitespace@84..85 " "
          Expression@85..106
            FunctionInvocation@85..106
              IdentGroup@85..103
                Ident@85..93 "DBMS_LOB"
                Dot@93..94 "."
                Ident@94..103 "GETLENGTH"
              LParen@103..104 "("
              ArgumentList@104..105
                Argument@104..105
                  IdentGroup@104..105
                    Ident@104..105 "a"
              RParen@105..106 ")"
          Semicolon@106..107 ";"
        Whitespace@107..108 "\n"
        Keyword@108..111 "END"
        Semicolon@111..112 ";"
      Whitespace@112..113 "\n"
  SelectStmt@113..150
    Keyword@113..119 "SELECT"
    Whitespace@119..120 " "
    SelectClause@120..140
      ColumnExpr@120..140
        FunctionInvocation@120..139
          IdentGroup@120..131
            Ident@120..131 "text_length"
          LParen@131..132 "("
          ArgumentList@132..138
            Argument@132..138
              QuotedLiteral@132..138 "'hans'"
          RParen@138..139 ")"
        Whitespace@139..140 " "
    Keyword@140..144 "FROM"
    Whitespace@144..145 " "
    IdentGroup@145..149
      Ident@145..149 "DUAL"
    Semicolon@149..150 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_cte_procedure() {
        check(
            parse(
                "WITH PROCEDURE print_contact(
    in_customer_id NUMBER 
)
IS
  r_contact contacts%ROWTYPE;
BEGIN
  -- get contact based on customer id
  SELECT *
  INTO r_contact
  FROM contacts
  WHERE customer_id = p_customer_id;

  -- print out contact's information
  dbms_output.put_line( r_contact.first_name || ' ' ||
  r_contact.last_name || '<' || r_contact.email ||'>' );

END; 
SELECT * from employee;",
                parse_cte,
            ),
            expect![[r#"
Root@0..397
  WithClause@0..374
    Keyword@0..4 "WITH"
    Whitespace@4..5 " "
    Procedure@5..374
      ProcedureHeader@5..59
        Keyword@5..14 "PROCEDURE"
        Whitespace@14..15 " "
        IdentGroup@15..28
          Ident@15..28 "print_contact"
        ParamList@28..58
          LParen@28..29 "("
          Whitespace@29..34 "\n    "
          Param@34..57
            IdentGroup@34..48
              Ident@34..48 "in_customer_id"
            Whitespace@48..49 " "
            Datatype@49..57
              Keyword@49..55 "NUMBER"
              Whitespace@55..57 " \n"
          RParen@57..58 ")"
        Whitespace@58..59 "\n"
      Keyword@59..61 "IS"
      Whitespace@61..64 "\n  "
      Block@64..372
        DeclareSection@64..92
          IdentGroup@64..73
            Ident@64..73 "r_contact"
          Whitespace@73..74 " "
          Datatype@74..90
            IdentGroup@74..82
              Ident@74..82 "contacts"
            TypeAttribute@82..90
              Percentage@82..83 "%"
              Keyword@83..90 "ROWTYPE"
          Semicolon@90..91 ";"
          Whitespace@91..92 "\n"
        Keyword@92..97 "BEGIN"
        Whitespace@97..100 "\n  "
        Comment@100..135 "-- get contact based  ..."
        Whitespace@135..138 "\n  "
        BlockStatement@138..257
          SelectStmt@138..216
            Keyword@138..144 "SELECT"
            Whitespace@144..145 " "
            Asterisk@145..146 "*"
            Whitespace@146..149 "\n  "
            IntoClause@149..166
              Keyword@149..153 "INTO"
              Whitespace@153..154 " "
              IdentGroup@154..163
                Ident@154..163 "r_contact"
              Whitespace@163..166 "\n  "
            Keyword@166..170 "FROM"
            Whitespace@170..171 " "
            IdentGroup@171..179
              Ident@171..179 "contacts"
            Whitespace@179..182 "\n  "
            WhereClause@182..215
              Keyword@182..187 "WHERE"
              Whitespace@187..188 " "
              Expression@188..215
                IdentGroup@188..199
                  Ident@188..199 "customer_id"
                Whitespace@199..200 " "
                ComparisonOp@200..201 "="
                Whitespace@201..202 " "
                IdentGroup@202..215
                  Ident@202..215 "p_customer_id"
            Semicolon@215..216 ";"
          Whitespace@216..220 "\n\n  "
          Comment@220..254 "-- print out contact' ..."
          Whitespace@254..257 "\n  "
        BlockStatement@257..366
          FunctionInvocation@257..365
            IdentGroup@257..277
              Ident@257..268 "dbms_output"
              Dot@268..269 "."
              Ident@269..277 "put_line"
            LParen@277..278 "("
            Whitespace@278..279 " "
            ArgumentList@279..364
              Argument@279..364
                Expression@279..364
                  Expression@279..358
                    Expression@279..339
                      Expression@279..332
                        Expression@279..307
                          IdentGroup@279..299
                            Ident@279..288 "r_contact"
                            Dot@288..289 "."
                            Ident@289..299 "first_name"
                          Whitespace@299..300 " "
                          Concat@300..302 "||"
                          Whitespace@302..303 " "
                          QuotedLiteral@303..306 "' '"
                          Whitespace@306..307 " "
                        Concat@307..309 "||"
                        Whitespace@309..312 "\n  "
                        IdentGroup@312..331
                          Ident@312..321 "r_contact"
                          Dot@321..322 "."
                          Ident@322..331 "last_name"
                        Whitespace@331..332 " "
                      Concat@332..334 "||"
                      Whitespace@334..335 " "
                      QuotedLiteral@335..338 "'<'"
                      Whitespace@338..339 " "
                    Concat@339..341 "||"
                    Whitespace@341..342 " "
                    IdentGroup@342..357
                      Ident@342..351 "r_contact"
                      Dot@351..352 "."
                      Ident@352..357 "email"
                    Whitespace@357..358 " "
                  Concat@358..360 "||"
                  QuotedLiteral@360..363 "'>'"
                  Whitespace@363..364 " "
            RParen@364..365 ")"
          Semicolon@365..366 ";"
        Whitespace@366..368 "\n\n"
        Keyword@368..371 "END"
        Semicolon@371..372 ";"
      Whitespace@372..374 " \n"
  SelectStmt@374..397
    Keyword@374..380 "SELECT"
    Whitespace@380..381 " "
    Asterisk@381..382 "*"
    Whitespace@382..383 " "
    Keyword@383..387 "from"
    Whitespace@387..388 " "
    IdentGroup@388..396
      Ident@388..396 "employee"
    Semicolon@396..397 ";"
"#]],
            vec![],
        );
    }
}
