// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use crate::grammar::declare_section::parse_declare_section;
use crate::parser::Parser;
use inner_source_gen::syntax::SyntaxKind;
use inner_source_gen::T;

use super::*;

/// Parses a complete package.
/// Refer to https://docs.oracle.com/en/database/oracle/oracle-database/23/lnpls/CREATE-PACKAGE-BODY-statement.html#GUID-68526FF2-96A1-4F14-A10B-4DD3E1CD80BE
pub(crate) fn parse_package(p: &mut Parser) {
    p.start(SyntaxKind::Package);
    parse_header(p);
    parse_body(p);
    p.finish();
}

/// Parses the header of a package.
fn parse_header(p: &mut Parser) {
    p.expect(T![create]);
    if p.eat(T![or]) {
        p.expect(T![replace]);
    }

    p.eat_one_of(&[T![editionable], T![noneditionable]]);

    p.expect(T![package]);
    p.expect(T![body]);

    if p.eat(T![if]) {
        p.expect(T![not]);
        p.expect(T![exists]);
    }

    parse_ident(p, 1..2);

    if p.eat(T![sharing]) {
        p.expect(T![=]);
        p.expect_one_of(&[T![metadata], T![none]]);
    }

    p.expect_one_of(&[T![as], T![is]]);
}

/// Parses the body of a package.
fn parse_body(p: &mut Parser) {
    parse_declare_section(p, None);

    if p.eat(T![begin]) {
        safe_loop!(p, {
            parse_stmt(p);

            if p.at(T![exception]) || p.at(T![end]) {
                break;
            }
        });

        if p.eat(T![exception]) {
            p.expect(T![when]);
            if !p.eat(T![others]) {
                safe_loop!(p, {
                    parse_ident(p, 1..1);
                    if !p.eat(T![or]) {
                        break;
                    }
                });
            }
            p.expect(T![then]);
            safe_loop!(p, {
                parse_stmt(p);
                if p.at(T![end]) {
                    break;
                }
            });
        }
    }

    p.expect(T![end]);
    parse_ident(p, 0..1);
    p.expect(T![;]);
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn parse_trigger_header() {
        check(
            parse(
                r#"CREATE OR REPLACE EDITIONABLE PACKAGE BODY IF NOT EXISTS my_schema.my_package SHARING = METADATA AS"#,
                parse_header,
            ),
            expect![[r#"
Root@0..99
  Keyword@0..6 "CREATE"
  Whitespace@6..7 " "
  Keyword@7..9 "OR"
  Whitespace@9..10 " "
  Keyword@10..17 "REPLACE"
  Whitespace@17..18 " "
  Keyword@18..29 "EDITIONABLE"
  Whitespace@29..30 " "
  Keyword@30..37 "PACKAGE"
  Whitespace@37..38 " "
  Keyword@38..42 "BODY"
  Whitespace@42..43 " "
  Keyword@43..45 "IF"
  Whitespace@45..46 " "
  Keyword@46..49 "NOT"
  Whitespace@49..50 " "
  Keyword@50..56 "EXISTS"
  Whitespace@56..57 " "
  IdentGroup@57..77
    Ident@57..66 "my_schema"
    Dot@66..67 "."
    Ident@67..77 "my_package"
  Whitespace@77..78 " "
  Keyword@78..85 "SHARING"
  Whitespace@85..86 " "
  ComparisonOp@86..87 "="
  Whitespace@87..88 " "
  Keyword@88..96 "METADATA"
  Whitespace@96..97 " "
  Keyword@97..99 "AS"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_util_package() {
        const INPUT: &str = include_str!("../../tests/package/util.ora.sql");
        check(
            parse(INPUT, parse_package),
            expect![[r#"
Root@0..153
  Package@0..152
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..14 "PACKAGE"
    Whitespace@14..15 " "
    Keyword@15..19 "BODY"
    Whitespace@19..20 " "
    IdentGroup@20..34
      Ident@20..29 "northwind"
      Dot@29..30 "."
      Ident@30..34 "util"
    Whitespace@34..35 " "
    Keyword@35..37 "AS"
    Whitespace@37..42 "\n    "
    DeclareSection@42..143
      Procedure@42..143
        ProcedureHeader@42..72
          Keyword@42..51 "PROCEDURE"
          Whitespace@51..52 " "
          IdentGroup@52..57
            Ident@52..57 "print"
          ParamList@57..71
            LParen@57..58 "("
            Param@58..70
              IdentGroup@58..61
                Ident@58..61 "str"
              Whitespace@61..62 " "
              Datatype@62..70
                Keyword@62..70 "varchar2"
            RParen@70..71 ")"
          Whitespace@71..72 " "
        Keyword@72..74 "IS"
        Whitespace@74..79 "\n    "
        Block@79..142
          Keyword@79..84 "BEGIN"
          Whitespace@84..93 "\n        "
          BlockStatement@93..133
            FunctionInvocation@93..132
              IdentGroup@93..113
                Ident@93..104 "DBMS_OUTPUT"
                Dot@104..105 "."
                Ident@105..113 "PUT_LINE"
              LParen@113..114 "("
              ArgumentList@114..131
                Argument@114..131
                  Expression@114..131
                    QuotedLiteral@114..124 "'Output: '"
                    Whitespace@124..125 " "
                    Concat@125..127 "||"
                    Whitespace@127..128 " "
                    IdentGroup@128..131
                      Ident@128..131 "str"
              RParen@131..132 ")"
            Semicolon@132..133 ";"
          Whitespace@133..138 "\n    "
          Keyword@138..141 "END"
          Semicolon@141..142 ";"
        Whitespace@142..143 "\n"
    Keyword@143..146 "END"
    Whitespace@146..147 " "
    IdentGroup@147..151
      Ident@147..151 "util"
    Semicolon@151..152 ";"
  Whitespace@152..153 "\n"
"#]],
            vec![],
        );
    }
}
