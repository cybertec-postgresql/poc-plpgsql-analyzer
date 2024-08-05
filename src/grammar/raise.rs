use crate::Parser;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

use super::parse_ident;

pub fn parse_raise_stmt(p: &mut Parser) {
    p.start(SyntaxKind::RaiseStmt);
    p.expect(T![raise]);
    if !p.at(T![;]) {
        parse_ident(p, 1..1);
    }
    p.eat(T![;]);
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::tests::{check, parse};

    use super::parse_raise_stmt;

    #[test]
    fn test_raise_exception_stmt() {
        check(
            parse(r#"RAISE i_am_tired;"#, parse_raise_stmt),
            expect![[r#"
Root@0..17
  RaiseStmt@0..17
    Keyword@0..5 "RAISE"
    Whitespace@5..6 " "
    IdentGroup@6..16
      Ident@6..16 "i_am_tired"
    Semicolon@16..17 ";"
"#]],
            vec![],
        );
    }
}
