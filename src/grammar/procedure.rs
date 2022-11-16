// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements parsing of procedures from a token tree.

use super::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

/// Parses a complete procedure.
pub(crate) fn parse_procedure(p: &mut Parser) {
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
    if p.eat(TokenKind::OrKw) {
        p.expect(TokenKind::ReplaceKw);
    }

    p.expect(TokenKind::ProcedureKw);

    parse_ident(p);
    parse_param_list(p);
    p.finish();
}

/// Parses the body of a procedure.
fn parse_body(p: &mut Parser) {
    p.expect_one_of(&[TokenKind::IsKw, TokenKind::AsKw]);
    p.eat(TokenKind::DollarQuote);
    parse_var_decl_list(p);
    p.expect(TokenKind::BeginKw);
    p.eat_ws();

    p.start(SyntaxKind::ProcedureBody);
    p.until_last(TokenKind::EndKw);
    p.finish();

    p.expect(TokenKind::EndKw);
    eat_ident(p);
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
    Keyword@7..9 "OR"
    Whitespace@9..10 " "
    Keyword@10..17 "REPLACE"
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
        Keyword@82..87 "%type"
        Whitespace@87..93 "\n     "
      Comma@93..94 ","
      Whitespace@94..95 " "
      Param@95..145
        Ident@95..107 "p_start_date"
        Whitespace@107..113 "      "
        Ident@113..135 "job_history.start_date"
        Keyword@135..140 "%type"
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

    #[test]
    fn test_parse_pg_procedure() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.pg.sql");

        check(
            parse(INPUT, parse_procedure),
            expect![[r#"
Root@0..304
  Procedure@0..304
    ProcedureHeader@0..30
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..16 "PROCEDURE"
      Whitespace@16..17 " "
      Ident@17..27 "secure_dml"
      ParamList@27..29
        LParen@27..28 "("
        RParen@28..29 ")"
      Whitespace@29..30 "\n"
    Keyword@30..32 "AS"
    Whitespace@32..33 " "
    DollarQuote@33..35 "$$"
    Whitespace@35..36 "\n"
    Keyword@36..41 "BEGIN"
    Whitespace@41..44 "\n  "
    ProcedureBody@44..278
      Ident@44..46 "IF"
      Whitespace@46..47 " "
      Ident@47..54 "TO_CHAR"
      Whitespace@54..55 " "
      LParen@55..56 "("
      Ident@56..63 "SYSDATE"
      Comma@63..64 ","
      Whitespace@64..65 " "
      QuotedLiteral@65..74 "'HH24:MI'"
      RParen@74..75 ")"
      Whitespace@75..76 " "
      Ident@76..79 "NOT"
      Whitespace@79..80 " "
      Ident@80..87 "BETWEEN"
      Whitespace@87..88 " "
      QuotedLiteral@88..95 "'08:00'"
      Whitespace@95..96 " "
      Keyword@96..99 "AND"
      Whitespace@99..100 " "
      QuotedLiteral@100..107 "'18:00'"
      Whitespace@107..116 "\n        "
      Keyword@116..118 "OR"
      Whitespace@118..119 " "
      Ident@119..126 "TO_CHAR"
      Whitespace@126..127 " "
      LParen@127..128 "("
      Ident@128..135 "SYSDATE"
      Comma@135..136 ","
      Whitespace@136..137 " "
      QuotedLiteral@137..141 "'DY'"
      RParen@141..142 ")"
      Whitespace@142..143 " "
      Keyword@143..145 "IN"
      Whitespace@145..146 " "
      LParen@146..147 "("
      QuotedLiteral@147..152 "'SAT'"
      Comma@152..153 ","
      Whitespace@153..154 " "
      QuotedLiteral@154..159 "'SUN'"
      RParen@159..160 ")"
      Whitespace@160..161 " "
      Ident@161..165 "THEN"
      Whitespace@165..170 "\n    "
      Ident@170..193 "RAISE_APPLICATION_ERROR"
      Whitespace@193..194 " "
      LParen@194..195 "("
      Integer@195..201 "-20205"
      Comma@201..202 ","
      Whitespace@202..211 "\n        "
      QuotedLiteral@211..265 "'You may only make ch ..."
      RParen@265..266 ")"
      SemiColon@266..267 ";"
      Whitespace@267..270 "\n  "
      Keyword@270..273 "END"
      Whitespace@273..274 " "
      Ident@274..276 "IF"
      SemiColon@276..277 ";"
      Whitespace@277..278 "\n"
    Keyword@278..281 "END"
    SemiColon@281..282 ";"
    Whitespace@282..283 "\n"
    DollarQuote@283..285 "$$"
    Whitespace@285..286 " "
    Ident@286..294 "LANGUAGE"
    Whitespace@294..295 " "
    Ident@295..302 "plpgsql"
    SemiColon@302..303 ";"
    Whitespace@303..304 "\n"
"#]],
        );
    }
}
