// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of functions from a token tree.

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

use super::*;

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

    p.eat_one_of(&[TokenKind::Editionable, TokenKind::NonEditionable]);

    p.expect(TokenKind::FunctionKw);

    parse_ident(p, 1..2);

    parse_param_list(p);
    parse_return_type(p);
    parse_attributes(p);
    parse_param_list(p);
    p.finish();
}

fn parse_return_type(p: &mut Parser) {
    if p.eat(TokenKind::ReturnKw) {
        parse_datatype(p);
    }
}

fn parse_attributes(p: &mut Parser) {
    p.eat(TokenKind::DeterministicKw);
}

/// Parses the body of a function.
fn parse_body(p: &mut Parser) {
    p.expect_one_of(&[TokenKind::IsKw, TokenKind::AsKw]);
    p.eat(TokenKind::DollarQuote);
    parse_var_decl_list(p);
    p.expect(TokenKind::BeginKw);
    p.eat_ws();

    p.start(SyntaxKind::FunctionBody);
    p.until_last(TokenKind::EndKw);
    p.finish();

    p.expect(TokenKind::EndKw);
    parse_ident(p, 0..1);
    p.expect(TokenKind::SemiColon);
    p.eat_ws();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

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
    IdentGroup@16..21
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
    IdentGroup@27..31
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
    IdentGroup@17..32
      Ident@17..32 "add_job_history"
    Whitespace@32..37 "\n    "
    ParamList@37..145
      LParen@37..38 "("
      Whitespace@38..40 "  "
      Param@40..92
        IdentGroup@40..48
          Ident@40..48 "p_emp_id"
        Whitespace@48..58 "          "
        Datatype@58..92
          IdentGroup@58..81
            Ident@58..69 "job_history"
            Dot@69..70 "."
            Ident@70..81 "employee_id"
          TypeAttribute@81..86
            Percentage@81..82 "%"
            Keyword@82..86 "type"
          Whitespace@86..92 "\n     "
      Comma@92..93 ","
      Whitespace@93..94 " "
      Param@94..144
        IdentGroup@94..106
          Ident@94..106 "p_start_date"
        Whitespace@106..112 "      "
        Datatype@112..144
          IdentGroup@112..134
            Ident@112..123 "job_history"
            Dot@123..124 "."
            Ident@124..134 "start_date"
          TypeAttribute@134..139
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
  IdentGroup@24..29
    Ident@24..29 "hello"
  SemiColon@29..30 ";"
  Whitespace@30..31 "\n"
"#]],
        );
    }

    #[test]
    fn test_editionable_function() {
        const INPUT: &str = include_str!("../../tests/function/heading/ignore_editionable.ora.sql");

        check(
            parse(INPUT, parse_function),
            expect![[r#"
Root@0..171
  Function@0..171
    FunctionHeader@0..146
      Comment@0..73 "-- test: ignore EDITI ..."
      Whitespace@73..74 "\n"
      Keyword@74..80 "CREATE"
      Whitespace@80..81 " "
      Keyword@81..83 "OR"
      Whitespace@83..84 " "
      Keyword@84..91 "REPLACE"
      Whitespace@91..92 " "
      Keyword@92..103 "EDITIONABLE"
      Whitespace@103..104 " "
      Keyword@104..112 "FUNCTION"
      Whitespace@112..113 " "
      IdentGroup@113..131
        Ident@113..131 "ignore_editionable"
      Whitespace@131..132 "\n"
      Keyword@132..138 "RETURN"
      Datatype@138..146
        Whitespace@138..139 " "
        Keyword@139..145 "number"
        Whitespace@145..146 " "
    Keyword@146..148 "IS"
    Whitespace@148..149 "\n"
    Keyword@149..154 "BEGIN"
    Whitespace@154..156 "\n "
    FunctionBody@156..166
      Keyword@156..162 "RETURN"
      Whitespace@162..163 " "
      Integer@163..164 "1"
      SemiColon@164..165 ";"
      Whitespace@165..166 "\n"
    Keyword@166..169 "END"
    SemiColon@169..170 ";"
    Whitespace@170..171 "\n"
"#]],
        );
    }

    #[test]
    fn test_non_editionable_function() {
        const INPUT: &str =
            include_str!("../../tests/function/heading/ignore_noneditionable.ora.sql");

        check(
            parse(INPUT, parse_function),
            expect![[r#"
Root@0..180
  Function@0..180
    FunctionHeader@0..155
      Comment@0..76 "-- test: ignore NONED ..."
      Whitespace@76..77 "\n"
      Keyword@77..83 "CREATE"
      Whitespace@83..84 " "
      Keyword@84..86 "OR"
      Whitespace@86..87 " "
      Keyword@87..94 "REPLACE"
      Whitespace@94..95 " "
      Keyword@95..109 "NONEDITIONABLE"
      Whitespace@109..110 " "
      Keyword@110..118 "FUNCTION"
      Whitespace@118..119 " "
      IdentGroup@119..140
        Ident@119..140 "ignore_noneditionable"
      Whitespace@140..141 "\n"
      Keyword@141..147 "RETURN"
      Datatype@147..155
        Whitespace@147..148 " "
        Keyword@148..154 "number"
        Whitespace@154..155 " "
    Keyword@155..157 "IS"
    Whitespace@157..158 "\n"
    Keyword@158..163 "BEGIN"
    Whitespace@163..165 "\n "
    FunctionBody@165..175
      Keyword@165..171 "RETURN"
      Whitespace@171..172 " "
      Integer@172..173 "1"
      SemiColon@173..174 ";"
      Whitespace@174..175 "\n"
    Keyword@175..178 "END"
    SemiColon@178..179 ";"
    Whitespace@179..180 "\n"
"#]],
        );
    }
}
