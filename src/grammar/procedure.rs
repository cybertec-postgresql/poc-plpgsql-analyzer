// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@ferrous-systems.com>

//! Implements parsing of procedures from a token tree.

use crate::grammar::call_spec::opt_call_spec;
use crate::parser::Parser;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;

use super::*;

/// Parses a complete procedure.
pub(crate) fn parse_procedure(p: &mut Parser, is_nested: bool) {
    p.start(SyntaxKind::Procedure);
    parse_header(p, is_nested);
    parse_body(p);
    p.finish();
}

/// Parses the header of a procedure.
fn parse_header(p: &mut Parser, is_nested: bool) {
    p.start(SyntaxKind::ProcedureHeader);

    if !is_nested {
        p.expect(T![create]);
        if p.eat(T![or]) {
            p.expect(T![replace]);
        }

        p.eat_one_of(&[T![editionable], T![noneditionable]]);
    }

    p.expect(T![procedure]);

    parse_ident(p, 1..2);
    parse_param_list(p);
    p.finish();
}

/// Parses the body of a procedure.
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

    use crate::ParseError;
    use crate::ParseErrorType::ExpectedToken;
    use source_gen::lexer::TokenKind::ProcedureKw;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_parse_header_without_replace() {
        check(
            parse("CREATE PROCEDURE hello", |p| parse_header(p, false)),
            expect![[r#"
Root@0..22
  ProcedureHeader@0..22
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..16 "PROCEDURE"
    Whitespace@16..17 " "
    IdentGroup@17..22
      Ident@17..22 "hello"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_invalid_header() {
        check(
            parse("CREATE hello", |p| parse_header(p, false)),
            expect![[r#"
Root@0..12
  ProcedureHeader@0..12
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    IdentGroup@7..12
      Ident@7..12 "hello"
"#]],
            vec![ParseError::new(ExpectedToken(ProcedureKw), 7..12)],
        );
    }

    #[test]
    fn test_parse_header_without_params() {
        const INPUT: &str = "CREATE OR REPLACE PROCEDURE test";
        check(
            parse(INPUT, |p| parse_header(p, false)),
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
    IdentGroup@28..32
      Ident@28..32 "test"
"#]],
            vec![],
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
            parse(INPUT, |p| parse_header(p, false)),
            expect![[r#"
Root@0..146
  Whitespace@0..1 "\n"
  ProcedureHeader@1..146
    Keyword@1..7 "CREATE"
    Whitespace@7..8 " "
    Keyword@8..17 "PROCEDURE"
    Whitespace@17..18 " "
    IdentGroup@18..33
      Ident@18..33 "add_job_history"
    Whitespace@33..38 "\n    "
    ParamList@38..146
      LParen@38..39 "("
      Whitespace@39..41 "  "
      Param@41..93
        IdentGroup@41..49
          Ident@41..49 "p_emp_id"
        Whitespace@49..59 "          "
        Datatype@59..93
          IdentGroup@59..82
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
        IdentGroup@95..107
          Ident@95..107 "p_start_date"
        Whitespace@107..113 "      "
        Datatype@113..145
          IdentGroup@113..135
            Ident@113..124 "job_history"
            Dot@124..125 "."
            Ident@125..135 "start_date"
          TypeAttribute@135..140
            Percentage@135..136 "%"
            Keyword@136..140 "type"
          Whitespace@140..145 "\n    "
      RParen@145..146 ")"
"#]],
            vec![],
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
  Block@4..30
    Keyword@4..9 "BEGIN"
    Whitespace@9..14 "\n    "
    BlockStatement@14..19
      Keyword@14..18 "NULL"
      Semicolon@18..19 ";"
    Whitespace@19..20 "\n"
    Keyword@20..23 "END"
    Whitespace@23..24 " "
    IdentGroup@24..29
      Ident@24..29 "hello"
    Semicolon@29..30 ";"
  Whitespace@30..31 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_proc_with_quoted_ident() {
        const INPUT: &str = include_str!("../../tests/fixtures/unicode_characters.ora.sql");
        check(
            parse(INPUT, |p| parse_procedure(p, false)),
            expect![[r#"
Root@0..98
  Procedure@0..98
    ProcedureHeader@0..41
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..16 "PROCEDURE"
      Whitespace@16..17 " "
      IdentGroup@17..40
        Ident@17..40 "\"读文👩🏼\u{200d}🔬\""
      Whitespace@40..41 "\n"
    Keyword@41..43 "IS"
    Whitespace@43..44 " "
    Block@44..97
      Keyword@44..49 "BEGIN"
      Whitespace@49..52 "\n  "
      BlockStatement@52..57
        Keyword@52..56 "NULL"
        Semicolon@56..57 ";"
      Whitespace@57..58 " "
      Comment@58..68 "-- メ メ"
      Whitespace@68..69 "\n"
      Keyword@69..72 "END"
      Whitespace@72..73 " "
      IdentGroup@73..96
        Ident@73..96 "\"读文👩🏼\u{200d}🔬\""
      Semicolon@96..97 ";"
    Whitespace@97..98 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_procedure_with_schema_qualifier() {
        const INPUT: &str = include_str!("../../tests/procedure/heading/schema_qualified.ora.sql");
        check(
            parse(INPUT, |p| parse_procedure(p, false)),
            expect![[r#"
Root@0..124
  Comment@0..58 "-- test: Qualify the  ..."
  Whitespace@58..59 "\n"
  Procedure@59..124
    ProcedureHeader@59..100
      Keyword@59..65 "CREATE"
      Whitespace@65..66 " "
      Keyword@66..75 "PROCEDURE"
      Whitespace@75..76 " "
      IdentGroup@76..99
        Ident@76..94 "\"alternate_SCHEMA\""
        Dot@94..95 "."
        Ident@95..99 "proc"
      Whitespace@99..100 "\n"
    Keyword@100..102 "IS"
    Whitespace@102..103 "\n"
    Block@103..123
      Keyword@103..108 "BEGIN"
      Whitespace@108..113 "\n    "
      BlockStatement@113..118
        Keyword@113..117 "NULL"
        Semicolon@117..118 ";"
      Whitespace@118..119 "\n"
      Keyword@119..122 "END"
      Semicolon@122..123 ";"
    Whitespace@123..124 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_pg_procedure() {
        const INPUT: &str = include_str!("../../tests/fixtures/secure_dml.pg.sql");

        check(
            parse(INPUT, |p| parse_procedure(p, false)),
            expect![[r#"
Root@0..304
  Procedure@0..304
    ProcedureHeader@0..30
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..16 "PROCEDURE"
      Whitespace@16..17 " "
      IdentGroup@17..27
        Ident@17..27 "secure_dml"
      ParamList@27..29
        LParen@27..28 "("
        RParen@28..29 ")"
      Whitespace@29..30 "\n"
    Keyword@30..32 "AS"
    Whitespace@32..33 " "
    DollarQuote@33..35 "$$"
    Whitespace@35..36 "\n"
    Block@36..282
      Keyword@36..41 "BEGIN"
      Whitespace@41..44 "\n  "
      BlockStatement@44..277
        Keyword@44..46 "IF"
        Whitespace@46..47 " "
        Expression@47..161
          Expression@47..116
            FunctionInvocation@47..75
              IdentGroup@47..54
                Ident@47..54 "TO_CHAR"
              Whitespace@54..55 " "
              LParen@55..56 "("
              ArgumentList@56..74
                Argument@56..63
                  IdentGroup@56..63
                    Ident@56..63 "SYSDATE"
                Comma@63..64 ","
                Whitespace@64..65 " "
                Argument@65..74
                  QuotedLiteral@65..74 "'HH24:MI'"
              RParen@74..75 ")"
            Whitespace@75..76 " "
            Keyword@76..79 "NOT"
            Whitespace@79..80 " "
            Keyword@80..87 "BETWEEN"
            Whitespace@87..88 " "
            QuotedLiteral@88..95 "'08:00'"
            Whitespace@95..96 " "
            Keyword@96..99 "AND"
            Whitespace@99..100 " "
            QuotedLiteral@100..107 "'18:00'"
            Whitespace@107..116 "\n        "
          LogicOp@116..118 "OR"
          Whitespace@118..119 " "
          Expression@119..161
            FunctionInvocation@119..142
              IdentGroup@119..126
                Ident@119..126 "TO_CHAR"
              Whitespace@126..127 " "
              LParen@127..128 "("
              ArgumentList@128..141
                Argument@128..135
                  IdentGroup@128..135
                    Ident@128..135 "SYSDATE"
                Comma@135..136 ","
                Whitespace@136..137 " "
                Argument@137..141
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
        Keyword@161..165 "THEN"
        Whitespace@165..170 "\n    "
        BlockStatement@170..267
          FunctionInvocation@170..266
            IdentGroup@170..193
              Ident@170..193 "RAISE_APPLICATION_ERROR"
            Whitespace@193..194 " "
            LParen@194..195 "("
            ArgumentList@195..265
              Argument@195..201
                Integer@195..201 "-20205"
              Comma@201..202 ","
              Whitespace@202..211 "\n        "
              Argument@211..265
                QuotedLiteral@211..265 "'You may only make ch ..."
            RParen@265..266 ")"
          Semicolon@266..267 ";"
        Whitespace@267..270 "\n  "
        Keyword@270..273 "END"
        Whitespace@273..274 " "
        Keyword@274..276 "IF"
        Semicolon@276..277 ";"
      Whitespace@277..278 "\n"
      Keyword@278..281 "END"
      Semicolon@281..282 ";"
    Whitespace@282..283 "\n"
    DollarQuote@283..285 "$$"
    Whitespace@285..286 " "
    Keyword@286..294 "LANGUAGE"
    Whitespace@294..295 " "
    Keyword@295..302 "plpgsql"
    Semicolon@302..303 ";"
    Whitespace@303..304 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_editionable_procedure() {
        const INPUT: &str =
            include_str!("../../tests/procedure/heading/ignore_editionable.ora.sql");

        check(
            parse(INPUT, |p| parse_procedure(p, false)),
            expect![[r#"
Root@0..176
  Comment@0..73 "-- test: ignore EDITI ..."
  Whitespace@73..74 "\n"
  Procedure@74..176
    ProcedureHeader@74..133
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
      IdentGroup@114..132
        Ident@114..132 "ignore_editionable"
      Whitespace@132..133 "\n"
    Keyword@133..135 "IS"
    Whitespace@135..136 "\n"
    Block@136..175
      Keyword@136..141 "BEGIN"
      Whitespace@141..146 "\n    "
      BlockStatement@146..151
        Keyword@146..150 "NULL"
        Semicolon@150..151 ";"
      Whitespace@151..152 "\n"
      Keyword@152..155 "END"
      Whitespace@155..156 " "
      IdentGroup@156..174
        Ident@156..174 "ignore_editionable"
      Semicolon@174..175 ";"
    Whitespace@175..176 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_non_editionable_procedure() {
        const INPUT: &str =
            include_str!("../../tests/procedure/heading/ignore_noneditionable.ora.sql");

        check(
            parse(INPUT, |p| parse_procedure(p, false)),
            expect![[r#"
Root@0..193
  Comment@0..81 "-- test: ignore NONED ..."
  Whitespace@81..82 "\n"
  Procedure@82..193
    ProcedureHeader@82..147
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
      IdentGroup@125..146
        Ident@125..146 "ignore_noneditionable"
      Whitespace@146..147 "\n"
    Keyword@147..149 "IS"
    Whitespace@149..150 "\n"
    Block@150..192
      Keyword@150..155 "BEGIN"
      Whitespace@155..160 "\n    "
      BlockStatement@160..165
        Keyword@160..164 "NULL"
        Semicolon@164..165 ";"
      Whitespace@165..166 "\n"
      Keyword@166..169 "END"
      Whitespace@169..170 " "
      IdentGroup@170..191
        Ident@170..191 "ignore_noneditionable"
      Semicolon@191..192 ";"
    Whitespace@192..193 "\n"
"#]],
            vec![],
        );
    }
}
