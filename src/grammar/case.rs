use crate::{
    grammar::parse_expr,
    parser::{safe_loop, Parser},
};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

pub(crate) fn parse_case(p: &mut Parser) {
    p.start(SyntaxKind::CaseStmt);
    p.expect(T![case]);
    if p.at(T![when]) {
        parse_searched_case_expression(p);
    } else {
        parse_simple_case_expression(p);
    }

    if p.at(T![else]) {
        parse_else_expression(p);
    }
    p.eat(T![end]);
    p.eat(T![;]);
    p.finish();
}

fn parse_searched_case_expression(p: &mut Parser) {
    p.start(SyntaxKind::SearchedCaseExpression);
    safe_loop!(p, {
        p.expect(T![when]);
        parse_expr(p);
        p.expect(T![then]);
        parse_expr(p);
        if [T![else], T![end]].contains(&p.current()) {
            break;
        }
    });
    p.finish();
}

fn parse_simple_case_expression(p: &mut Parser) {
    p.start(SyntaxKind::SimpleCaseExpression);
    parse_expr(p);
    safe_loop!(p, {
        p.expect(T![when]);
        parse_comparison_expr(p);
        p.expect(T![then]);
        parse_expr(p);
        if [T![else], T![end]].contains(&p.current()) {
            break;
        }
    });
    p.finish();
}

fn parse_comparison_expr(p: &mut Parser) {
    p.start(SyntaxKind::ComparissonExpression);
    parse_expr(p);
    p.finish();
}

fn parse_else_expression(p: &mut Parser) {
    p.start(SyntaxKind::ElseExpression);
    p.expect(T![else]);
    parse_expr(p);
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;
    use crate::grammar::tests::{check, parse};

    #[test]
    fn parse_simple_case_clause() {
        check(
            parse(
                "CASE credit_limit WHEN 100 THEN 'Low'
WHEN 5000 THEN 'High'
ELSE 'Medium' END",
                parse_case,
            ),
            expect![[r#"
Root@0..77
  CaseStmt@0..77
    Keyword@0..4 "CASE"
    Whitespace@4..5 " "
    SimpleCaseExpression@5..60
      IdentGroup@5..17
        Ident@5..17 "credit_limit"
      Whitespace@17..18 " "
      Keyword@18..22 "WHEN"
      Whitespace@22..23 " "
      ComparissonExpression@23..27
        Integer@23..26 "100"
        Whitespace@26..27 " "
      Keyword@27..31 "THEN"
      Whitespace@31..32 " "
      QuotedLiteral@32..37 "'Low'"
      Whitespace@37..38 "\n"
      Keyword@38..42 "WHEN"
      Whitespace@42..43 " "
      ComparissonExpression@43..48
        Integer@43..47 "5000"
        Whitespace@47..48 " "
      Keyword@48..52 "THEN"
      Whitespace@52..53 " "
      QuotedLiteral@53..59 "'High'"
      Whitespace@59..60 "\n"
    ElseExpression@60..74
      Keyword@60..64 "ELSE"
      Whitespace@64..65 " "
      QuotedLiteral@65..73 "'Medium'"
      Whitespace@73..74 " "
    Keyword@74..77 "END"
"#]],
            vec![],
        );
    }
}
