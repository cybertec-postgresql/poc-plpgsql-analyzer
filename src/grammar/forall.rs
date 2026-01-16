use crate::Parser;

use super::{parse_dml, parse_expr, parse_ident};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

pub(crate) fn parse_forall_stmt(p: &mut Parser) {
    p.start(SyntaxKind::ForallStmt);
    p.expect(T![forall]);
    parse_ident(p, 1..2);
    p.expect(T![in]);
    parse_bounds_clause(p);
    if p.eat(T![save]) {
        p.expect(T![exceptions]);
    }
    parse_dml(p);
    p.eat(T![;]);
    p.finish();
}

pub(crate) fn parse_bounds_clause(p: &mut Parser) {
    p.start(SyntaxKind::BoundsClause);
    match p.current() {
        T![values] => {
            p.expect(T![values]);
            p.expect(T![of]);
            parse_ident(p, 1..2);
        }
        T![indices] => {
            p.expect(T![indices]);
            p.expect(T![of]);
            parse_ident(p, 1..2);
            if p.eat(T![between]) {
                parse_expr(p);
                p.expect(T![and]);
                parse_expr(p);
            }
        }
        _ => {
            parse_expr(p);
            p.expect(T![iter_range]);
            parse_expr(p);
        }
    }
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::tests::{check, parse};

    use super::*;

    #[test]
    fn test_parse_forall() {
        check(
            parse(
                "FORALL i IN depts.FIRST..depts.LAST
    DELETE FROM employees_temp
    WHERE department_id = depts(i);",
                parse_forall_stmt,
            ),
            expect![[r#"
Root@0..102
  ForallStmt@0..102
    Keyword@0..6 "FORALL"
    Whitespace@6..7 " "
    IdentGroup@7..8
      Ident@7..8 "i"
    Whitespace@8..9 " "
    Keyword@9..11 "IN"
    Whitespace@11..12 " "
    BoundsClause@12..40
      IdentGroup@12..23
        Ident@12..17 "depts"
        Dot@17..18 "."
        Ident@18..23 "FIRST"
      IterRange@23..25 ".."
      IdentGroup@25..35
        Ident@25..30 "depts"
        Dot@30..31 "."
        Ident@31..35 "LAST"
      Whitespace@35..40 "\n    "
    DeleteStmt@40..102
      Keyword@40..46 "DELETE"
      Whitespace@46..47 " "
      Keyword@47..51 "FROM"
      Whitespace@51..52 " "
      IdentGroup@52..66
        Ident@52..66 "employees_temp"
      Whitespace@66..71 "\n    "
      WhereClause@71..101
        Keyword@71..76 "WHERE"
        Whitespace@76..77 " "
        Expression@77..101
          IdentGroup@77..90
            Ident@77..90 "department_id"
          Whitespace@90..91 " "
          ComparisonOp@91..92 "="
          Whitespace@92..93 " "
          FunctionInvocation@93..101
            IdentGroup@93..98
              Ident@93..98 "depts"
            LParen@98..99 "("
            ArgumentList@99..100
              Argument@99..100
                IdentGroup@99..100
                  Ident@99..100 "i"
            RParen@100..101 ")"
      Semicolon@101..102 ";"
"#]],
            vec![],
        );
    }
}
