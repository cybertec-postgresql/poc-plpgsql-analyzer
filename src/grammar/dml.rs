use super::{parse_ident, parse_where_clause};
use crate::grammar::parse_expr;
use crate::parser::Parser;
use crate::safe_loop;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

#[allow(unused)]
pub(crate) fn parse_dml(p: &mut Parser) {
    if p.at(T![delete]) {
        parse_delete(p);
    } else {
        parse_update(p);
    }
}

pub(crate) fn parse_delete(p: &mut Parser) {
    p.start(SyntaxKind::DeleteStmt);
    p.expect(T![delete]);
    p.expect(T![from]);
    parse_ident(p, 1..2);
    parse_where_clause(p);
    p.eat(T![;]);
    p.finish();
}

pub(crate) fn parse_update(p: &mut Parser) {
    p.start(SyntaxKind::UpdateStmt);
    p.expect(T![update]);
    p.expect_one_of(&[T![unquoted_ident], T![quoted_ident]]);
    parse_set_clause(p);
    parse_where_clause(p);
    p.eat(T![;]);
    p.finish();
}

pub(crate) fn parse_set_clause(p: &mut Parser) {
    p.start(SyntaxKind::SetClause);
    p.expect(T![set]);
    safe_loop!(p, {
        parse_expr(p);
        if p.at(T![where]) {
            break;
        }
        p.eat(T![,]);
    });
    p.finish()
}

#[cfg(test)]
mod tests {
    use super::super::tests::{check, parse};
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_parse_simple_delete() {
        check(
            parse("DELETE FROM emp WHERE emp_id = 69;", parse_dml),
            expect![[r#"
Root@0..34
  DeleteStmt@0..34
    Keyword@0..6 "DELETE"
    Whitespace@6..7 " "
    Keyword@7..11 "FROM"
    Whitespace@11..12 " "
    IdentGroup@12..15
      Ident@12..15 "emp"
    Whitespace@15..16 " "
    WhereClause@16..33
      Keyword@16..21 "WHERE"
      Whitespace@21..22 " "
      Expression@22..33
        IdentGroup@22..28
          Ident@22..28 "emp_id"
        Whitespace@28..29 " "
        ComparisonOp@29..30 "="
        Whitespace@30..31 " "
        Integer@31..33 "69"
    Semicolon@33..34 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_simple_update() {
        check(
            parse(
                "UPDATE emp SET salary = salary*2 WHERE emp_firstname=Jeremy;",
                parse_dml,
            ),
            expect![[r#"
Root@0..60
  UpdateStmt@0..60
    Keyword@0..6 "UPDATE"
    Whitespace@6..7 " "
    Ident@7..10 "emp"
    Whitespace@10..11 " "
    SetClause@11..33
      Keyword@11..14 "SET"
      Whitespace@14..15 " "
      Expression@15..33
        IdentGroup@15..21
          Ident@15..21 "salary"
        Whitespace@21..22 " "
        ComparisonOp@22..23 "="
        Whitespace@23..24 " "
        Expression@24..33
          IdentGroup@24..30
            Ident@24..30 "salary"
          ArithmeticOp@30..31 "*"
          Integer@31..32 "2"
          Whitespace@32..33 " "
    WhereClause@33..59
      Keyword@33..38 "WHERE"
      Whitespace@38..39 " "
      Expression@39..59
        IdentGroup@39..52
          Ident@39..52 "emp_firstname"
        ComparisonOp@52..53 "="
        IdentGroup@53..59
          Ident@53..59 "Jeremy"
    Semicolon@59..60 ";"
"#]],
            vec![],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{check, parse};
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_parse_simple_delete() {
        check(
            parse("DELETE FROM emp WHERE emp_id = 69;", |p| parse_dml(p)),
            expect![[r#"
Root@0..34
  DeleteStmt@0..34
    Keyword@0..6 "DELETE"
    Whitespace@6..7 " "
    Keyword@7..11 "FROM"
    Whitespace@11..12 " "
    Ident@12..15 "emp"
    Whitespace@15..16 " "
    WhereClause@16..33
      Keyword@16..21 "WHERE"
      Whitespace@21..22 " "
      Expression@22..33
        IdentGroup@22..28
          Ident@22..28 "emp_id"
        Whitespace@28..29 " "
        ComparisonOp@29..30 "="
        Whitespace@30..31 " "
        Integer@31..33 "69"
    Semicolon@33..34 ";"
"#]],
            vec![],
        );
    }
}
