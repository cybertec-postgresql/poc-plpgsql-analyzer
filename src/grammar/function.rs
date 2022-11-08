// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of functions from a token tree.

use super::{parse_ident, parse_param_list, parse_typename};
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

/// Parses a complete function.
pub fn parse_function(p: &mut Parser) {
    p.start(SyntaxKind::Function);
    parse_header(p);
    parse_body(p);
    while !p.at(TokenKind::Eof) {
        p.bump_any();
    }
    p.finish();
}

/// Parses the header of a function.
fn parse_header(p: &mut Parser) {
    p.start(SyntaxKind::FunctionHeader);
    p.expect(TokenKind::CreateKw);
    if p.eat(TokenKind::OrKw) {
        p.expect(TokenKind::ReplaceKw);
    }

    p.expect(TokenKind::FunctionKw);

    parse_ident(p);
    parse_param_list(p);
    parse_return_type(p);
    parse_attributes(p);
    parse_param_list(p);
    p.finish();
}

fn parse_return_type(p: &mut Parser) {
    if p.eat(TokenKind::ReturnKw) {
        parse_typename(p);
    }
}

fn parse_attributes(p: &mut Parser) {
    p.eat(TokenKind::DeterministicKw);
}

/// Parses the body of a function.
fn parse_body(p: &mut Parser) {
    p.expect(TokenKind::IsKw);
    p.expect(TokenKind::BeginKw);
    p.eat_ws();

    p.start(SyntaxKind::FunctionBody);
    p.until_last(TokenKind::EndKw);
    p.finish();

    p.expect(TokenKind::EndKw);
    parse_ident(p);
    p.expect(TokenKind::SemiColon);
    p.eat_ws();
}

#[cfg(test)]
mod tests {
    use super::super::tests::{check, parse};
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_parse_header_without_replace() {
        check(
            parse("CREATE FUNCTION hello", parse_header),
            expect![[r#"
Root@0..21
  FunctionHeader@0..21
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..15 "FUNCTION"
    Whitespace@15..16 " "
    Ident@16..21 "hello"
"#]],
        );
    }

    #[test]
    fn test_parse_header_without_params() {
        const INPUT: &str = "CREATE OR REPLACE FUNCTION test";
        check(
            parse(INPUT, parse_header),
            expect![[r#"
Root@0..31
  FunctionHeader@0..31
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..9 "OR"
    Whitespace@9..10 " "
    Keyword@10..17 "REPLACE"
    Whitespace@17..18 " "
    Keyword@18..26 "FUNCTION"
    Whitespace@26..27 " "
    Ident@27..31 "test"
"#]],
        );
    }

    #[test]
    fn test_parse_header_with_params() {
        const INPUT: &str = r#"
CREATE FUNCTION add_job_history
    (  p_emp_id          job_history.employee_id%type
     , p_start_date      job_history.start_date%type
    )"#;
        check(
            parse(INPUT, parse_header),
            expect![[r#"
Root@0..145
  FunctionHeader@0..145
    Whitespace@0..1 "\n"
    Keyword@1..7 "CREATE"
    Whitespace@7..8 " "
    Keyword@8..16 "FUNCTION"
    Whitespace@16..17 " "
    Ident@17..32 "add_job_history"
    Whitespace@32..37 "\n    "
    ParamList@37..145
      LParen@37..38 "("
      Whitespace@38..40 "  "
      Param@40..92
        Ident@40..48 "p_emp_id"
        Whitespace@48..58 "          "
        Ident@58..81 "job_history.employee_id"
        Percentage@81..82 "%"
        Keyword@82..86 "type"
        Whitespace@86..92 "\n     "
      Comma@92..93 ","
      Whitespace@93..94 " "
      Param@94..144
        Ident@94..106 "p_start_date"
        Whitespace@106..112 "      "
        Ident@112..134 "job_history.start_date"
        Percentage@134..135 "%"
        Keyword@135..139 "type"
        Whitespace@139..144 "\n    "
      RParen@144..145 ")"
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
  FunctionBody@14..20
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
