// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements parsing of procedures from a token tree.

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

use super::*;

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

    p.eat_one_of(&[TokenKind::Editionable, TokenKind::NonEditionable]);

    p.expect(TokenKind::ProcedureKw);

    parse_qualified_ident(p, 1..2);
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
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

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
        Datatype@59..93
          QualifiedIdent@59..82
            Ident@59..70 "job_history"
            Dot@70..71 "."
            Ident@71..82 "employee_id"
          TypeAttribute@82..87
            Percentage@82..83 "%"
            Keyword@83..87 "type"
          Whitespace@87..93 "\n     "
      Comma@93..94 ","
      Whitespace@94..95 " "
      Param@95..145
        Ident@95..107 "p_start_date"
        Whitespace@107..113 "      "
        Datatype@113..145
          QualifiedIdent@113..135
            Ident@113..124 "job_history"
            Dot@124..125 "."
            Ident@125..135 "start_date"
          TypeAttribute@135..140
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

    #[test]
    fn test_parse_proc_with_quoted_ident() {
        const INPUT: &str = include_str!("../../tests/fixtures/unicode_characters.ora.sql");
        check(
            parse(INPUT, parse_procedure),
            expect![[r#"
Root@0..98
  Procedure@0..98
    ProcedureHeader@0..41
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..16 "PROCEDURE"
      Whitespace@16..17 " "
      Ident@17..40 "\"读文👩🏼\u{200d}🔬\""
      Whitespace@40..41 "\n"
    Keyword@41..43 "IS"
    Whitespace@43..44 " "
    Keyword@44..49 "BEGIN"
    Whitespace@49..52 "\n  "
    ProcedureBody@52..69
      Ident@52..56 "NULL"
      SemiColon@56..57 ";"
      Whitespace@57..58 " "
      Comment@58..68 "-- メ メ"
      Whitespace@68..69 "\n"
    Keyword@69..72 "END"
    Whitespace@72..73 " "
    Ident@73..96 "\"读文👩🏼\u{200d}🔬\""
    SemiColon@96..97 ";"
    Whitespace@97..98 "\n"
"#]],
        );
    }

    #[test]
    fn test_parse_procedure_with_schema_qualifier() {
        const INPUT: &str = include_str!("../../tests/procedure/heading/schema_qualified.ora.sql");
        check(
            parse(INPUT, parse_procedure),
            expect![[r#"
Root@0..124
  Procedure@0..124
    ProcedureHeader@0..100
      Comment@0..58 "-- test: Qualify the  ..."
      Whitespace@58..59 "\n"
      Keyword@59..65 "CREATE"
      Whitespace@65..66 " "
      Keyword@66..75 "PROCEDURE"
      QualifiedIdent@75..99
        Whitespace@75..76 " "
        Ident@76..94 "\"alternate_SCHEMA\""
        Dot@94..95 "."
        Ident@95..99 "proc"
      Whitespace@99..100 "\n"
    Keyword@100..102 "IS"
    Whitespace@102..103 "\n"
    Keyword@103..108 "BEGIN"
    Whitespace@108..113 "\n    "
    ProcedureBody@113..119
      Ident@113..117 "NULL"
      SemiColon@117..118 ";"
      Whitespace@118..119 "\n"
    Keyword@119..122 "END"
    SemiColon@122..123 ";"
    Whitespace@123..124 "\n"
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
      Keyword@76..79 "NOT"
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

    #[test]
    fn test_editionable_procedure() {
        const INPUT: &str =
            include_str!("../../tests/procedure/heading/ignore_editionable.ora.sql");

        check(
            parse(INPUT, parse_procedure),
            expect![[r#"
Root@0..176
  Procedure@0..176
    ProcedureHeader@0..133
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
      Keyword@104..113 "PROCEDURE"
      Whitespace@113..114 " "
      Ident@114..132 "ignore_editionable"
      Whitespace@132..133 "\n"
    Keyword@133..135 "IS"
    Whitespace@135..136 "\n"
    Keyword@136..141 "BEGIN"
    Whitespace@141..146 "\n    "
    ProcedureBody@146..152
      Ident@146..150 "NULL"
      SemiColon@150..151 ";"
      Whitespace@151..152 "\n"
    Keyword@152..155 "END"
    Whitespace@155..156 " "
    Ident@156..174 "ignore_editionable"
    SemiColon@174..175 ";"
    Whitespace@175..176 "\n"
"#]],
        );
    }

    #[test]
    fn test_non_editionable_procedure() {
        const INPUT: &str =
            include_str!("../../tests/procedure/heading/ignore_noneditionable.ora.sql");

        check(
            parse(INPUT, parse_procedure),
            expect![[r#"
Root@0..193
  Procedure@0..193
    ProcedureHeader@0..147
      Comment@0..81 "-- test: ignore NONED ..."
      Whitespace@81..82 "\n"
      Keyword@82..88 "CREATE"
      Whitespace@88..89 " "
      Keyword@89..91 "OR"
      Whitespace@91..92 " "
      Keyword@92..99 "REPLACE"
      Whitespace@99..100 " "
      Keyword@100..114 "NONEDITIONABLE"
      Whitespace@114..115 " "
      Keyword@115..124 "PROCEDURE"
      Whitespace@124..125 " "
      Ident@125..146 "ignore_noneditionable"
      Whitespace@146..147 "\n"
    Keyword@147..149 "IS"
    Whitespace@149..150 "\n"
    Keyword@150..155 "BEGIN"
    Whitespace@155..160 "\n    "
    ProcedureBody@160..166
      Ident@160..164 "NULL"
      SemiColon@164..165 ";"
      Whitespace@165..166 "\n"
    Keyword@166..169 "END"
    Whitespace@169..170 " "
    Ident@170..191 "ignore_noneditionable"
    SemiColon@191..192 ";"
    Whitespace@192..193 "\n"
"#]],
        );
    }
}
