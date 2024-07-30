// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of functions from a token tree.

use crate::grammar::call_spec::opt_call_spec;
use crate::parser::Parser;
use inner_source_gen::lexer::TokenKind;
use inner_source_gen::syntax::SyntaxKind;

use super::*;

/// Parses a complete function.
pub fn parse_function(p: &mut Parser, is_nested: bool) {
    p.start(SyntaxKind::Function);
    parse_header(p, is_nested);
    parse_body(p);
    p.finish();
}

/// Parses the header of a function.
fn parse_header(p: &mut Parser, is_nested: bool) {
    p.start(SyntaxKind::FunctionHeader);

    if !is_nested {
        p.expect(T![create]);
        if p.eat(T![or]) {
            p.expect(T![replace]);
        }

        p.eat_one_of(&[T![editionable], T![noneditionable]]);
    }

    p.expect(T![function]);

    parse_ident(p, 1..2);

    parse_param_list(p);
    parse_return_type(p);
    parse_attributes(p);
    parse_param_list(p);
    p.finish();
}

fn parse_return_type(p: &mut Parser) {
    if p.eat(T![return]) {
        parse_datatype(p);
    }
}

fn parse_attributes(p: &mut Parser) {
    p.eat(T![deterministic]);
}

/// Parses the body of a function.
fn parse_body(p: &mut Parser) {
    p.expect_one_of(&[T![is], T![as]]);
    p.eat(T!["$$"]);

    if !opt_call_spec(p) {
        parse_block(p);
    }

    p.eat(T!["$$"]);
    p.eat(T![language]);
    p.eat(T![plpgsql]);
    p.eat(T![;]);
    p.eat(T![/]);
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_parse_header_without_replace() {
        check(
            parse("CREATE FUNCTION hello", |p| parse_header(p, false)),
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
            vec![],
        );
    }

    #[test]
    fn test_parse_header_without_params() {
        const INPUT: &str = "CREATE OR REPLACE FUNCTION test";
        check(
            parse(INPUT, |p| parse_header(p, false)),
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
            vec![],
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
            parse(INPUT, |p| parse_header(p, false)),
            expect![[r#"
Root@0..145
  Whitespace@0..1 "\n"
  FunctionHeader@1..145
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
            vec![],
        );
    }

    #[test]
    fn test_parse_body() {
        check(
            parse(r#"IS BEGIN NULL; END hello;"#, parse_body),
            expect![[r#"
Root@0..25
  Keyword@0..2 "IS"
  Whitespace@2..3 " "
  Block@3..25
    Keyword@3..8 "BEGIN"
    Whitespace@8..9 " "
    BlockStatement@9..14
      Keyword@9..13 "NULL"
      Semicolon@13..14 ";"
    Whitespace@14..15 " "
    Keyword@15..18 "END"
    Whitespace@18..19 " "
    IdentGroup@19..24
      Ident@19..24 "hello"
    Semicolon@24..25 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_editionable_function() {
        const INPUT: &str = include_str!("../../tests/function/heading/ignore_editionable.ora.sql");

        check(
            parse(INPUT, |p| parse_function(p, false)),
            expect![[r#"
Root@0..171
  Comment@0..73 "-- test: ignore EDITI ..."
  Whitespace@73..74 "\n"
  Function@74..171
    FunctionHeader@74..146
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
      Whitespace@138..139 " "
      Datatype@139..146
        Keyword@139..145 "number"
        Whitespace@145..146 " "
    Keyword@146..148 "IS"
    Whitespace@148..149 "\n"
    Block@149..170
      Keyword@149..154 "BEGIN"
      Whitespace@154..156 "\n "
      BlockStatement@156..165
        Keyword@156..162 "RETURN"
        Whitespace@162..163 " "
        Expression@163..164
          Integer@163..164 "1"
        Semicolon@164..165 ";"
      Whitespace@165..166 "\n"
      Keyword@166..169 "END"
      Semicolon@169..170 ";"
    Whitespace@170..171 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_non_editionable_function() {
        const INPUT: &str =
            include_str!("../../tests/function/heading/ignore_noneditionable.ora.sql");

        check(
            parse(INPUT, |p| parse_function(p, false)),
            expect![[r#"
Root@0..180
  Comment@0..76 "-- test: ignore NONED ..."
  Whitespace@76..77 "\n"
  Function@77..180
    FunctionHeader@77..155
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
      Whitespace@147..148 " "
      Datatype@148..155
        Keyword@148..154 "number"
        Whitespace@154..155 " "
    Keyword@155..157 "IS"
    Whitespace@157..158 "\n"
    Block@158..179
      Keyword@158..163 "BEGIN"
      Whitespace@163..165 "\n "
      BlockStatement@165..174
        Keyword@165..171 "RETURN"
        Whitespace@171..172 " "
        Expression@172..173
          Integer@172..173 "1"
        Semicolon@173..174 ";"
      Whitespace@174..175 "\n"
      Keyword@175..178 "END"
      Semicolon@178..179 ";"
    Whitespace@179..180 "\n"
"#]],
            vec![],
        );
    }
}
