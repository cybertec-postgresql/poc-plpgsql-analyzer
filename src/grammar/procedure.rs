use crate::{lexer::TokenKind, parser::Parser, SyntaxKind, Token};

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
    p.eat_ws();
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
    p.start(SyntaxKind::ProcedureBody);
    p.eat_ws();
    p.expect(TokenKind::IsKw);
    p.eat_ws();
    p.expect(TokenKind::BeginKw);
    p.finish();
}

/// Parses the parameter list in the procedure header
fn parse_param_list(p: &mut Parser) {
    p.eat_ws();
    if let Some(TokenKind::LParen) = p.peek() {
        p.start(SyntaxKind::ParamList);
        p.consume();

        loop {
            p.eat_ws();
            match p.peek() {
                Some(TokenKind::Comma) => {
                    p.consume();
                }
                Some(TokenKind::RParen) => {
                    p.consume();
                    break;
                }
                Some(_) => {
                    parse_param(p);
                }
                None => {
                    p.error(TokenKind::RParen);
                    break;
                }
            }
        }

        p.finish();
    }
}

fn parse_param(p: &mut Parser) {
    p.start(SyntaxKind::Param);
    parse_ident(p);
    p.eat_ws();
    parse_param_type(p);
    p.finish();
}

fn parse_ident(p: &mut Parser) {
    p.eat_ws();
    if let Some(TokenKind::Ident) = p.peek() {
        p.consume();
    } else {
        p.error(TokenKind::Ident);
    }
}

fn parse_param_type(p: &mut Parser) {
    p.start(SyntaxKind::ParamType);
    parse_ident(p);
    if let Some(TokenKind::Percentage) = p.peek() {
        p.consume();
        p.expect(TokenKind::Ident);
    }
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::{
        grammar::procedure::{parse_header, parse_param},
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
    fn test_parse_procedure() {}

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
        check(
            parse("p_1 VARCHAR2", parse_param),
            expect![[r#"
Root@0..12
  Param@0..12
    Ident@0..3 "p_1"
    Whitespace@3..4 " "
    ParamType@4..12
      Ident@4..12 "VARCHAR2"
"#]],
        );

        check(
            parse("  foo bar%type", parse_param),
            expect![[r#"
Root@0..14
  Param@0..14
    Whitespace@0..2 "  "
    Ident@2..5 "foo"
    Whitespace@5..6 " "
    ParamType@6..14
      Ident@6..9 "bar"
      Percentage@9..10 "%"
      Ident@10..14 "type"
"#]],
        );
    }

    #[test]
    fn test_parse_ident_with_trivia() {
        const INPUT: &str = " -- hello\n  foo";
        check(
            parse(INPUT, parse_ident),
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
        check(
            parse("CREATE PROCEDURE hello", parse_header),
            expect![[r#"
Root@0..22
  ProcedureHeader@0..22
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..16 "PROCEDURE"
    Whitespace@16..17 " "
    Ident@17..22 "hello"
"#]],
        );
    }

    #[test]
    fn test_parse_invalid_header() {
        check(
            parse("CREATE hello", parse_header),
            expect![[r#"
Root@0..12
  ProcedureHeader@0..12
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Error@7..12
      Ident@7..12 "hello"
    Error@12..12
"#]],
        );
    }

    #[test]
    fn test_parse_header_without_params() {
        const INPUT: &str = "CREATE OR REPLACE PROCEDURE test";
        check(
            parse(INPUT, parse_header),
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
        const INPUT: &str = r#"
CREATE PROCEDURE add_job_history
    (  p_emp_id          job_history.employee_id%type
     , p_start_date      job_history.start_date%type
    )"#;
        check(
            parse(INPUT, parse_header),
            expect![[r#"
Root@0..146
  ProcedureHeader@0..146
    Whitespace@0..1 "\n"
    Keyword@1..7 "CREATE"
    Whitespace@7..8 " "
    Keyword@8..17 "PROCEDURE"
    Whitespace@17..18 " "
    Ident@18..33 "add_job_history"
    Whitespace@33..38 "\n    "
    ParamList@38..146
      LParen@38..39 "("
      Whitespace@39..41 "  "
      Param@41..87
        Ident@41..49 "p_emp_id"
        Whitespace@49..59 "          "
        ParamType@59..87
          Ident@59..82 "job_history.employee_id"
          Percentage@82..83 "%"
          Ident@83..87 "type"
      Whitespace@87..93 "\n     "
      Comma@93..94 ","
      Whitespace@94..95 " "
      Param@95..140
        Ident@95..107 "p_start_date"
        Whitespace@107..113 "      "
        ParamType@113..140
          Ident@113..135 "job_history.start_date"
          Percentage@135..136 "%"
          Ident@136..140 "type"
      Whitespace@140..145 "\n    "
      RParen@145..146 ")"
"#]],
        );
    }
}
