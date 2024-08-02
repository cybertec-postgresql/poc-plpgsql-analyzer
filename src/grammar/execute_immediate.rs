use crate::grammar::parse_ident;
use crate::parser::{safe_loop, Parser};
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

use super::parse_into_clause;

pub(crate) fn parse_execute_immediate(p: &mut Parser) {
    p.start(SyntaxKind::ExecuteImmediateStmt);
    p.expect(T![execute]);
    p.expect(T![immediate]);
    // Parse String
    if !p.eat(T![quoted_literal]) {
        parse_ident(p, 1..1);
    }
    if p.at(T![into]) {
        parse_into_clause(p, true);
    }
    if p.at(T![bulk]) {
        parse_bulk_into_clause(p);
    }
    if p.at(T![using]) {
        parse_using_clause(p);
    }
    if [T![return], T![returning]].contains(&p.current()) {
        parse_return_into_clause(p);
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_using_clause(p: &mut Parser) {
    p.start(SyntaxKind::UsingClause);
    p.expect(T![using]);
    safe_loop!(p, {
        if [T![in], T![out]].contains(&p.current()) {
            p.eat(T![in]);
            p.eat(T![out]);
        }
        parse_ident(p, 1..1);
        if [T![return], T![returning], T![;]].contains(&p.current()) {
            break;
        }
        p.eat(T![,]);
    });
    p.finish();
}

fn parse_return_into_clause(p: &mut Parser) {
    p.start(SyntaxKind::ReturnIntoClause);
    p.expect_one_of(&[T![return], T![returning]]);
    // Check if bulk into or normal into
    if p.at(T![bulk]) {
        parse_bulk_into_clause(p);
    } else {
        parse_into_clause(p, false);
    }
    p.finish();
}

fn parse_bulk_into_clause(p: &mut Parser) {
    p.start(SyntaxKind::BulkIntoClause);
    p.expect(T![bulk]);
    p.expect(T![collect]);
    p.expect(T![into]);
    safe_loop!(p, {
        if !p.eat(T![bind_var]) {
            parse_ident(p, 1..1);
        }
        if [T![using], T![;]].contains(&p.current()) {
            break;
        }
        p.eat(T![,]);
    });
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_parse_simple_execute_immediate() {
        check(
            parse(
                r#"EXECUTE IMMEDIATE 'SELECT * FROM emp;';"#,
                parse_execute_immediate,
            ),
            expect![[r#"
Root@0..39
  ExecuteImmediateStmt@0..39
    Keyword@0..7 "EXECUTE"
    Whitespace@7..8 " "
    Keyword@8..17 "IMMEDIATE"
    Whitespace@17..18 " "
    QuotedLiteral@18..38 "'SELECT * FROM emp;'"
    Semicolon@38..39 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_complex_execute_immediate() {
        check(
            parse(
                r#"EXECUTE IMMEDIATE sql_stmt USING emp_id RETURNING INTO salary;"#,
                parse_execute_immediate,
            ),
            expect![[r#"
Root@0..62
  ExecuteImmediateStmt@0..62
    Keyword@0..7 "EXECUTE"
    Whitespace@7..8 " "
    Keyword@8..17 "IMMEDIATE"
    Whitespace@17..18 " "
    IdentGroup@18..26
      Ident@18..26 "sql_stmt"
    Whitespace@26..27 " "
    UsingClause@27..40
      Keyword@27..32 "USING"
      Whitespace@32..33 " "
      IdentGroup@33..39
        Ident@33..39 "emp_id"
      Whitespace@39..40 " "
    ReturnIntoClause@40..61
      Keyword@40..49 "RETURNING"
      Whitespace@49..50 " "
      IntoClause@50..61
        Keyword@50..54 "INTO"
        Whitespace@54..55 " "
        IdentGroup@55..61
          Ident@55..61 "salary"
    Semicolon@61..62 ";"
"#]],
            vec![],
        );
    }
}
