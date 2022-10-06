use crate::{parser::Parser, Token, lexer::TokenKind, SyntaxKind};

/// Parses a complete procedure.
pub(crate) fn parse_procedure(p: &mut Parser) {
    p.start(SyntaxKind::Procedure);
    p.eat_ws();
    parse_header(p);
    parse_body(p);
    p.finish();
}

/// Parses the header
fn parse_header(p: &mut Parser) {
    p.start(SyntaxKind::ProcedureHeader);
    p.expect(TokenKind::CreateKw);
    p.eat_ws();
    if let Some(TokenKind::OrReplaceKw) = p.peek() {
        p.consume();
        p.eat_ws();
    }
    p.expect(TokenKind::ProcedureKw);
    parse_ident(p);
    parse_param_list(p);
    p.finish();
}

fn parse_body(p: &mut Parser) {

}

/// Parses the parameter list in the procedure header
fn parse_param_list(p: &mut Parser) {
    if let Some(TokenKind::LParen) = p.peek() {
        p.start(SyntaxKind::ParamList);
        p.consume();
        p.expect(TokenKind::RParen);
        p.finish();
    }
}

fn parse_ident(p: &mut Parser) {
    p.eat_ws();
    if let Some(TokenKind::Ident) = p.peek() {
        p.consume();
    } else {
        p.error(TokenKind::Ident);
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
        Lexer, grammar::procedure::parse_header,
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
    fn test_parse_param() {
        // const INPUT: &str = ""
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

    #[test]
    fn test_parse_header_without_replace() {
        check(parse("CREATE PROCEDURE hello", parse_header),
            expect![[r#"
Root@0..22
  ProcedureHeader@0..22
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..16 "PROCEDURE"
    Whitespace@16..17 " "
    Ident@17..22 "hello"
"#]]
        );
    }

    #[test]
    fn test_parse_header_without_params() {
        const INPUT: &str = "CREATE OR REPLACE PROCEDURE test";
        check(parse(INPUT, parse_header),
        expect![[r#"
Root@0..32
  ProcedureHeader@0..32
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..17 "OR REPLACE"
    Whitespace@17..18 " "
    Keyword@18..27 "PROCEDURE"
    Whitespace@27..28 " "
    Ident@28..32 "test"
"#]],
        );
    }

    #[test]
    fn test_parse_header_with_params() {
    }
}
