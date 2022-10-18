// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements parsing of procedures from a token tree.

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

/// Parses a complete procedure.
pub fn parse_procedure(p: &mut Parser) {
    p.start(SyntaxKind::Procedure);
    parse_header(p);
    parse_body(p);
    while !p.at(TokenKind::Eof) {
        p.bump_any();
    }
    p.finish();
}

/// Parses the header of a procedure.
fn parse_header(p: &mut Parser) {
    p.start(SyntaxKind::ProcedureHeader);
    p.expect(TokenKind::CreateKw);
    if p.at(TokenKind::OrReplaceKw) {
        p.bump(TokenKind::OrReplaceKw);
    }
    p.expect(TokenKind::ProcedureKw);
    parse_ident(p);
    parse_param_list(p);
    p.finish();
}

/// Parses the body of a procedure.
fn parse_body(p: &mut Parser) {
    p.expect_one_of(&[TokenKind::IsKw, TokenKind::AsKw]);
    p.expect(TokenKind::BeginKw);
    p.eat_ws();

    p.start(SyntaxKind::ProcedureBody);
    p.until_last(TokenKind::EndKw);
    p.finish();

    p.expect(TokenKind::EndKw);
    parse_ident(p);
    p.expect(TokenKind::SemiColon);
    p.eat_ws();
}

/// Parses the parameter list in the procedure header
fn parse_param_list(p: &mut Parser) {
    if p.at(TokenKind::LParen) {
        p.start(SyntaxKind::ParamList);
        p.bump(TokenKind::LParen);

        loop {
            match p.current() {
                TokenKind::Comma => {
                    p.bump(TokenKind::Comma);
                }
                TokenKind::RParen | TokenKind::Eof => {
                    break;
                }
                _ => {
                    parse_param(p);
                }
            }
        }

        p.expect(TokenKind::RParen);
        p.finish();
    }
}

/// Parses a single parameter in a parameter list of a procedure.
///
/// Example:
///   p2 VARCHAR2 := 'not empty'
fn parse_param(p: &mut Parser) {
    p.start(SyntaxKind::Param);
    p.expect(TokenKind::Ident);

    while !p.at(TokenKind::RParen) && !p.at(TokenKind::Comma) && !p.at(TokenKind::Eof) {
        p.bump_any();
    }

    p.finish();
}

/// Parses a single SQL identifier.
fn parse_ident(p: &mut Parser) {
    p.expect(TokenKind::Ident);
}

#[cfg(test)]
mod tests {
    use crate::grammar::procedure::{parse_body, parse_header, parse_param};
    use crate::lexer::Lexer;
    use crate::parser::{Parse, Parser};
    use expect_test::{expect, Expect};

    use super::parse_ident;

    /// Helper function to compare the build syntax tree with the expected output.
    fn check(parse: Parse, expected_tree: Expect) {
        expected_tree.assert_eq(parse.tree().as_str())
    }

    /// Creates a new parser by lexing the input first.
    fn build_parser(input: &str) -> Parser {
        let mut tokens = Lexer::new(input).collect::<Vec<_>>();
        tokens.reverse();
        Parser::new(tokens)
    }

    /// A helper to allow to call the different parse functions.
    fn parse<F>(input: &str, f: F) -> Parse
    where
        F: Fn(&mut Parser),
    {
        let mut parser = build_parser(input);
        f(&mut parser);
        parser.build()
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
        assert!(parse("p_1 VARCHAR2", parse_param).ok());
        assert!(parse("p_2 NUMBER", parse_param).ok());
        assert!(parse("p_3 IN BOOLEAN := FALSE", parse_param).ok());
        assert!(parse("p_4 IN OUT NOCOPY DATE", parse_param).ok());
        assert!(parse("p_5", parse_param).ok());

        check(
            parse("p_1 VARCHAR2", parse_param),
            expect![[r#"
Root@0..12
  Param@0..12
    Ident@0..3 "p_1"
    Whitespace@3..4 " "
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
    Ident@6..9 "bar"
    Percentage@9..10 "%"
    Ident@10..14 "type"
"#]],
        );
    }

    #[test]
    fn test_parse_param_with_default_value() {
        check(
            parse("p2 VARCHAR2 := 'not empty'", parse_param),
            expect![[r#"
Root@0..26
  Param@0..26
    Ident@0..2 "p2"
    Whitespace@2..3 " "
    Ident@3..11 "VARCHAR2"
    Whitespace@11..12 " "
    Assign@12..14 ":="
    Whitespace@14..15 " "
    QuotedLiteral@15..26 "'not empty'"
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
Root@0..40
  ProcedureHeader@0..40
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Error@7..35
      Text@7..35 "Expected token 'Proce ..."
    Ident@35..40 "hello"
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
      Param@41..93
        Ident@41..49 "p_emp_id"
        Whitespace@49..59 "          "
        Ident@59..82 "job_history.employee_id"
        Percentage@82..83 "%"
        Keyword@83..87 "type"
        Whitespace@87..93 "\n     "
      Comma@93..94 ","
      Whitespace@94..95 " "
      Param@95..145
        Ident@95..107 "p_start_date"
        Whitespace@107..113 "      "
        Ident@113..135 "job_history.start_date"
        Percentage@135..136 "%"
        Keyword@136..140 "type"
        Whitespace@140..145 "\n    "
      RParen@145..146 ")"
"#]],
        );
    }

    #[test]
    fn test_parse_body() {
        const INPUT: &str = r#"
IS
BEGIN
    NULL;
END hello;
"#;
        check(
            parse(INPUT, parse_body),
            expect![[r#"
Root@0..31
  Whitespace@0..1 "\n"
  Keyword@1..3 "IS"
  Whitespace@3..4 "\n"
  Keyword@4..9 "BEGIN"
  Whitespace@9..14 "\n    "
  ProcedureBody@14..20
    Ident@14..18 "NULL"
    SemiColon@18..19 ";"
    Whitespace@19..20 "\n"
  Keyword@20..23 "END"
  Whitespace@23..24 " "
  Ident@24..29 "hello"
  SemiColon@29..30 ";"
  Whitespace@30..31 "\n"
"#]],
        );
    }
}
