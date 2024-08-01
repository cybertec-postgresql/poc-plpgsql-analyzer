use super::parse_where_clause;
use crate::parser::Parser;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

#[allow(unused)]
pub(crate) fn parse_dml(p: &mut Parser) {
    parse_delete(p);
}

pub(crate) fn parse_delete(p: &mut Parser) {
    p.start(SyntaxKind::DeleteStmt);
    p.expect(T![delete]);
    p.expect(T![from]);
    p.expect_one_of(&[T![quoted_ident], T![unquoted_ident]]);
    parse_where_clause(p);

    p.eat(T![;]);
    p.finish();
}

#[cfg(test)]
mod tests {
    use super::super::tests::{check, parse};
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_parse_simple_delete() {
        check(
            parse("DELETE FROM emp WHERE emp_id = 69;", parse_dml(p)),
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
