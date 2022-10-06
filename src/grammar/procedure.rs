use crate::{parser::Parser, Token, lexer::TokenKind, SyntaxKind};

/// Parses a complete procedure.
pub(crate) fn parse_procedure(p: &mut Parser) {
    p.start(SyntaxKind::Procedure);
    p.finish();
}

/// Parses the parameter list in the procedure header
fn parse_param_list(p: &mut Parser) {
    p.eat_ws();
}

fn parse_ident(p: &mut Parser) {
    p.eat_ws();
    match p.peek() {
        Some(TokenKind::Ident) => { p.consume(); },
        _ => p.error(TokenKind::Ident),
    }
}

fn parse_comma(p: &mut Parser) {
    p.eat_ws();
    match p.peek() {
        Some(TokenKind::Comma) => { p.consume(); },
        _ => ()
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::{
        parser::{Parse, Parser},
        Lexer,
    };

    use super::parse_ident;

    fn check(parse: Parse, expected_tree: expect_test::Expect) {
        expected_tree.assert_eq(parse.tree().as_str())
    }

    fn parse<F>(input: &str, f: F) -> Parse
    where
        F: Fn(&mut Parser),
    {
        let mut tokens = Lexer::new(input).collect::<Vec<_>>();
        tokens.reverse();
        let mut parser = Parser::new(tokens);
        f(&mut parser);
        parser.build()
    }

    #[test]
    fn test_parse_procedure() {
    }

    #[test]
    fn test_parse_ident() {
        check(
            parse("hello", parse_ident),
            expect![[r#"
Root@0..5
  Ident@0..5 "hello"
"#]],
        );
    }

    #[test]
    fn test_parse_ident_with_trivia() {
        const INPUT: &str = " -- hello\n  foo";
        check(parse(INPUT, parse_ident),
        expect![[r#"
Root@0..15
  Whitespace@0..1 " "
  Comment@1..9 "-- hello"
  Whitespace@9..12 "\n  "
  Ident@12..15 "foo"
"#]],
        );
    }
}
