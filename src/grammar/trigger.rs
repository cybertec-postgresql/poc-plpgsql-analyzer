// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

use crate::parser::Parser;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;

use super::*;

/// Parses a complete trigger.
pub(crate) fn parse_trigger(p: &mut Parser) {
    p.start(SyntaxKind::Trigger);
    parse_header(p);
    parse_body(p);
    p.finish();
}

/// Parses the header of a trigger.
fn parse_header(p: &mut Parser) {
    p.start(SyntaxKind::TriggerHeader);
    p.expect(T![create]);
    if p.eat(T![or]) {
        p.expect(T![replace]);
    }

    p.eat_one_of(&[T![editionable], T![noneditionable]]);

    p.expect(T![trigger]);
    parse_ident(p, 1..2);

    match p.current() {
        T![before] | T![instead] | T![after] => {
            if p.eat(T![instead]) {
                p.expect(T![of]);
            } else {
                p.bump_any();
            }

            match p.current() {
                T![insert] | T![update] | T![delete] => parse_simple_dml_trigger(p),
                _ => parse_system_trigger(p),
            };
        }
        T![for] => {
            p.error(ParseErrorType::Unimplemented(
                "compound trigger".to_string(),
            ));
        }
        _ => {
            p.expect_one_of(&[T![before], T![instead], T![after], T![for]]);
        }
    }

    p.finish();
}

fn parse_simple_dml_trigger(p: &mut Parser) {
    parse_dml_event_clause(p);
    parse_referencing_clause(p);

    if p.eat(T![for]) {
        p.expect(T![each]);
        p.expect(T![row]);
    }

    parse_trigger_edition_clause(p);
    parse_trigger_ordering_clause(p);

    p.eat_one_of(&[T![enable], T![disable]]);

    if p.eat(T![when]) {
        p.expect(T!["("]);
        parse_expr(p);
        p.expect(T![")"]);
    }
}

fn parse_system_trigger(p: &mut Parser) {
    safe_loop!(p, {
        let bump_n = match [
            p.current(),
            p.nth(1).unwrap_or(T![EOF]),
            p.nth(2).unwrap_or(T![EOF]),
        ] {
            [T![alter], ..]
            | [T![analyze], ..]
            | [T![audit], ..]
            | [T![comment], ..]
            | [T![create], ..]
            | [T![ddl], ..]
            | [T![drop], ..]
            | [T![grant], ..]
            | [T![noaudit], ..]
            | [T![rename], ..]
            | [T![revoke], ..]
            | [T![truncate], ..] => 1,

            [T![after], T![clone], ..]
            | [T![after], T![db_role_change], ..]
            | [T![after], T![logon], ..]
            | [T![after], T![servererror], ..]
            | [T![after], T![startup], ..]
            | [T![after], T![suspend], ..]
            | [T![associate], T![statistics], ..]
            | [T![before], T![logoff], ..]
            | [T![before], T![shutdown], ..]
            | [T![before], T![unplug], ..]
            | [T![disassociate], T![statistics], ..] => 2,

            [T![before], T![set], T![container]] | [T![after], T![set], T![container]] => 3,
            _ => {
                p.error(ParseErrorType::ExpectedDdlOrDatabaseEvent);
                return;
            }
        };

        for _ in 0..bump_n {
            p.bump_any();
        }

        if !p.eat(T![or]) {
            break;
        }
    });

    p.expect(T![on]);

    if p.at(T![pluggable]) || p.at(T![database]) {
        p.eat(T![pluggable]);
        p.expect(T![database]);
    } else {
        parse_ident(p, 0..1);
        p.eat(T![.]);
        p.expect(T![schema]);
    }

    parse_trigger_ordering_clause(p);

    p.eat_one_of(&[T![enable], T![disable]]);
}

fn parse_dml_event_clause(p: &mut Parser) {
    safe_loop!(p, {
        let token = p.current();

        if !p.expect_one_of(&[T![insert], T![update], T![delete]]) {
            break;
        }

        if token == T![update] && p.eat(T![of]) {
            safe_loop!(p, {
                parse_ident(p, 1..1);
                if !p.eat(T![,]) {
                    break;
                }
            });
        }

        if !p.eat(T![or]) {
            break;
        }
    });
    p.expect(T![on]);
    parse_ident(p, 1..2);
}

const REFERENCING_TOKENS: &[TokenKind] = &[T![old], T![new], T![parent]];
fn parse_referencing_clause(p: &mut Parser) {
    if p.eat(T![referencing]) {
        safe_loop!(p, {
            if !p.expect_one_of(REFERENCING_TOKENS) {
                break;
            }
            p.eat(T![as]);
            parse_ident(p, 1..1);

            if !REFERENCING_TOKENS.contains(&p.current()) {
                break;
            }
        });
    }
}

fn parse_trigger_edition_clause(p: &mut Parser) {
    if p.eat_one_of(&[T![forward], T![reverse]]) {
        p.expect(T![crossedition]);
    }
}

fn parse_trigger_ordering_clause(p: &mut Parser) {
    if p.eat_one_of(&[T![follows], T![precedes]]) {
        safe_loop!(p, {
            parse_ident(p, 1..2);
            if !p.eat(T![,]) {
                break;
            }
        });
    }
}

/// Parses the body of a trigger.
fn parse_body(p: &mut Parser) {
    if p.eat(T![call]) {
        parse_function_invocation(p);
    } else {
        parse_block(p);
    }
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
                r#"CREATE OR REPLACE EDITIONABLE TRIGGER my_schema.my_trigger
                AFTER INSERT OR UPDATE OF my_col, my_col_2
                ON my_table
                FOR EACH ROW"#,
                parse_header,
            ),
            expect![[r#"
Root@0..174
  TriggerHeader@0..174
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..9 "OR"
    Whitespace@9..10 " "
    Keyword@10..17 "REPLACE"
    Whitespace@17..18 " "
    Keyword@18..29 "EDITIONABLE"
    Whitespace@29..30 " "
    Keyword@30..37 "TRIGGER"
    Whitespace@37..38 " "
    IdentGroup@38..58
      Ident@38..47 "my_schema"
      Dot@47..48 "."
      Ident@48..58 "my_trigger"
    Whitespace@58..75 "\n                "
    Keyword@75..80 "AFTER"
    Whitespace@80..81 " "
    Keyword@81..87 "INSERT"
    Whitespace@87..88 " "
    Keyword@88..90 "OR"
    Whitespace@90..91 " "
    Keyword@91..97 "UPDATE"
    Whitespace@97..98 " "
    Keyword@98..100 "OF"
    Whitespace@100..101 " "
    IdentGroup@101..107
      Ident@101..107 "my_col"
    Comma@107..108 ","
    Whitespace@108..109 " "
    IdentGroup@109..117
      Ident@109..117 "my_col_2"
    Whitespace@117..134 "\n                "
    Keyword@134..136 "ON"
    Whitespace@136..137 " "
    IdentGroup@137..145
      Ident@137..145 "my_table"
    Whitespace@145..162 "\n                "
    Keyword@162..165 "FOR"
    Whitespace@165..166 " "
    Keyword@166..170 "EACH"
    Whitespace@170..171 " "
    Keyword@171..174 "ROW"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_header_with_referencing_and_edition_and_ordering_clause() {
        check(
            parse(
                r#"CREATE TRIGGER my_trigger
                    BEFORE INSERT ON my_table
                    REFERENCING OLD alt NEW AS neu
                    FORWARD CROSSEDITION
                    FOLLOWS my_schema.my_trigger_1, my_trigger_2"#,
                parse_header,
            ),
            expect![[r#"
Root@0..228
  TriggerHeader@0..228
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..14 "TRIGGER"
    Whitespace@14..15 " "
    IdentGroup@15..25
      Ident@15..25 "my_trigger"
    Whitespace@25..46 "\n                    "
    Keyword@46..52 "BEFORE"
    Whitespace@52..53 " "
    Keyword@53..59 "INSERT"
    Whitespace@59..60 " "
    Keyword@60..62 "ON"
    Whitespace@62..63 " "
    IdentGroup@63..71
      Ident@63..71 "my_table"
    Whitespace@71..92 "\n                    "
    Keyword@92..103 "REFERENCING"
    Whitespace@103..104 " "
    Keyword@104..107 "OLD"
    Whitespace@107..108 " "
    IdentGroup@108..111
      Ident@108..111 "alt"
    Whitespace@111..112 " "
    Keyword@112..115 "NEW"
    Whitespace@115..116 " "
    Keyword@116..118 "AS"
    Whitespace@118..119 " "
    IdentGroup@119..122
      Ident@119..122 "neu"
    Whitespace@122..143 "\n                    "
    Keyword@143..150 "FORWARD"
    Whitespace@150..151 " "
    Keyword@151..163 "CROSSEDITION"
    Whitespace@163..184 "\n                    "
    Keyword@184..191 "FOLLOWS"
    Whitespace@191..192 " "
    IdentGroup@192..214
      Ident@192..201 "my_schema"
      Dot@201..202 "."
      Ident@202..214 "my_trigger_1"
    Comma@214..215 ","
    Whitespace@215..216 " "
    IdentGroup@216..228
      Ident@216..228 "my_trigger_2"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_after_trigger() {
        const INPUT: &str = include_str!("../../tests/trigger/after_trigger.ora.sql");
        check(
            parse(INPUT, parse_trigger),
            expect![[r#"
Root@0..237
  Trigger@0..237
    TriggerHeader@0..123
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..9 "OR"
      Whitespace@9..10 " "
      Keyword@10..17 "REPLACE"
      Whitespace@17..18 " "
      Keyword@18..29 "EDITIONABLE"
      Whitespace@29..30 " "
      Keyword@30..37 "TRIGGER"
      Whitespace@37..38 " "
      IdentGroup@38..57
        Ident@38..43 "store"
        Dot@43..44 "."
        Ident@44..57 "after_trigger"
      Whitespace@57..60 "\n  "
      Keyword@60..65 "AFTER"
      Whitespace@65..66 " "
      Keyword@66..72 "UPDATE"
      Whitespace@72..73 " "
      Keyword@73..75 "OF"
      Whitespace@75..76 " "
      IdentGroup@76..84
        Ident@76..84 "order_id"
      Comma@84..85 ","
      Whitespace@85..86 " "
      IdentGroup@86..94
        Ident@86..94 "group_id"
      Whitespace@94..95 " "
      Keyword@95..97 "ON"
      Whitespace@97..98 " "
      IdentGroup@98..107
        Ident@98..107 "customers"
      Whitespace@107..110 "\n  "
      Keyword@110..113 "FOR"
      Whitespace@113..114 " "
      Keyword@114..118 "EACH"
      Whitespace@118..119 " "
      Keyword@119..122 "ROW"
      Whitespace@122..123 "\n"
    Block@123..236
      Keyword@123..128 "BEGIN"
      Whitespace@128..131 "\n  "
      BlockStatement@131..231
        FunctionInvocation@131..230
          IdentGroup@131..142
            Ident@131..142 "add_history"
          LParen@142..143 "("
          ArgumentList@143..229
            Argument@143..159
              Expression@143..159
                IdentGroup@143..159
                  BindVar@143..147 ":old"
                  Dot@147..148 "."
                  Ident@148..159 "customer_id"
            Comma@159..160 ","
            Whitespace@160..161 " "
            Argument@161..176
              Expression@161..176
                IdentGroup@161..176
                  BindVar@161..165 ":old"
                  Dot@165..166 "."
                  Ident@166..176 "created_at"
            Comma@176..177 ","
            Whitespace@177..178 " "
            Argument@178..185
              Expression@178..185
                IdentGroup@178..185
                  Ident@178..185 "sysdate"
            Comma@185..186 ","
            Whitespace@186..201 "\n              "
            Argument@201..214
              Expression@201..214
                IdentGroup@201..214
                  BindVar@201..205 ":old"
                  Dot@205..206 "."
                  Ident@206..214 "order_id"
            Comma@214..215 ","
            Whitespace@215..216 " "
            Argument@216..229
              IdentGroup@216..229
                BindVar@216..220 ":old"
                Dot@220..221 "."
                Ident@221..229 "group_id"
          RParen@229..230 ")"
        Semicolon@230..231 ";"
      Whitespace@231..232 "\n"
      Keyword@232..235 "END"
      Semicolon@235..236 ";"
    Whitespace@236..237 "\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_instead_of_trigger() {
        const INPUT: &str = include_str!("../../tests/trigger/instead_of_trigger.ora.sql");
        check(
            parse(INPUT, parse_trigger),
            expect![[r#"
Root@0..518
  Trigger@0..518
    TriggerHeader@0..95
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..9 "OR"
      Whitespace@9..10 " "
      Keyword@10..17 "REPLACE"
      Whitespace@17..18 " "
      Keyword@18..25 "TRIGGER"
      Whitespace@25..26 " "
      IdentGroup@26..44
        Ident@26..44 "instead_of_trigger"
      Whitespace@44..49 "\n    "
      Keyword@49..56 "INSTEAD"
      Whitespace@56..57 " "
      Keyword@57..59 "OF"
      Whitespace@59..60 " "
      Keyword@60..66 "INSERT"
      Whitespace@66..67 " "
      Keyword@67..69 "ON"
      Whitespace@69..70 " "
      IdentGroup@70..77
        Ident@70..77 "my_view"
      Whitespace@77..82 "\n    "
      Keyword@82..85 "FOR"
      Whitespace@85..86 " "
      Keyword@86..90 "EACH"
      Whitespace@90..91 " "
      Keyword@91..94 "ROW"
      Whitespace@94..95 "\n"
    Block@95..516
      DeclareSection@95..118
        Keyword@95..102 "DECLARE"
        Whitespace@102..107 "\n    "
        IdentGroup@107..109
          Ident@107..109 "id"
        Whitespace@109..110 " "
        Datatype@110..116
          Keyword@110..116 "NUMBER"
        Semicolon@116..117 ";"
        Whitespace@117..118 "\n"
      Keyword@118..123 "BEGIN"
      Whitespace@123..128 "\n    "
      BlockStatement@128..133
        Keyword@128..132 "NULL"
        Semicolon@132..133 ";"
      Whitespace@133..138 "\n    "
      Comment@138..168 "-- insert a new custo ..."
      Whitespace@168..173 "\n    "
      BlockStatement@173..368
        InsertStmt@173..336
          Keyword@173..179 "INSERT"
          Whitespace@179..180 " "
          Keyword@180..184 "INTO"
          Whitespace@184..185 " "
          IdentGroup@185..194
            Ident@185..194 "customers"
          LParen@194..195 "("
          IdentGroup@195..199
            Ident@195..199 "name"
          Comma@199..200 ","
          Whitespace@200..201 " "
          IdentGroup@201..208
            Ident@201..208 "address"
          Comma@208..209 ","
          Whitespace@209..210 " "
          IdentGroup@210..217
            Ident@210..217 "website"
          Comma@217..218 ","
          Whitespace@218..219 " "
          IdentGroup@219..231
            Ident@219..231 "credit_limit"
          RParen@231..232 ")"
          Whitespace@232..237 "\n    "
          Keyword@237..243 "VALUES"
          LParen@243..244 "("
          Expression@244..253
            IdentGroup@244..253
              BindVar@244..248 ":NEW"
              Dot@248..249 "."
              Ident@249..253 "NAME"
          Comma@253..254 ","
          Whitespace@254..255 " "
          Expression@255..267
            IdentGroup@255..267
              BindVar@255..259 ":NEW"
              Dot@259..260 "."
              Ident@260..267 "address"
          Comma@267..268 ","
          Whitespace@268..269 " "
          Expression@269..281
            IdentGroup@269..281
              BindVar@269..273 ":NEW"
              Dot@273..274 "."
              Ident@274..281 "website"
          Comma@281..282 ","
          Whitespace@282..283 " "
          IdentGroup@283..300
            BindVar@283..287 ":NEW"
            Dot@287..288 "."
            Ident@288..300 "credit_limit"
          RParen@300..301 ")"
          Whitespace@301..306 "\n    "
          Keyword@306..315 "RETURNING"
          Whitespace@315..316 " "
          IdentGroup@316..327
            Ident@316..327 "customer_id"
          Whitespace@327..328 " "
          Keyword@328..332 "INTO"
          Whitespace@332..333 " "
          IdentGroup@333..335
            Ident@333..335 "id"
          Semicolon@335..336 ";"
        Whitespace@336..342 "\n\n    "
        Comment@342..363 "-- insert the contact"
        Whitespace@363..368 "\n    "
      BlockStatement@368..512
        InsertStmt@368..511
          Keyword@368..374 "INSERT"
          Whitespace@374..375 " "
          Keyword@375..379 "INTO"
          Whitespace@379..380 " "
          IdentGroup@380..388
            Ident@380..388 "contacts"
          LParen@388..389 "("
          IdentGroup@389..399
            Ident@389..399 "first_name"
          Comma@399..400 ","
          Whitespace@400..401 " "
          IdentGroup@401..410
            Ident@401..410 "last_name"
          Comma@410..411 ","
          Whitespace@411..412 " "
          IdentGroup@412..417
            Ident@412..417 "email"
          Comma@417..418 ","
          Whitespace@418..419 " "
          IdentGroup@419..424
            Ident@419..424 "phone"
          Comma@424..425 ","
          Whitespace@425..426 " "
          IdentGroup@426..437
            Ident@426..437 "customer_id"
          RParen@437..438 ")"
          Whitespace@438..443 "\n    "
          Keyword@443..449 "VALUES"
          LParen@449..450 "("
          Expression@450..465
            IdentGroup@450..465
              BindVar@450..454 ":NEW"
              Dot@454..455 "."
              Ident@455..465 "first_name"
          Comma@465..466 ","
          Whitespace@466..467 " "
          Expression@467..481
            IdentGroup@467..481
              BindVar@467..471 ":NEW"
              Dot@471..472 "."
              Ident@472..481 "last_name"
          Comma@481..482 ","
          Whitespace@482..483 " "
          Expression@483..493
            IdentGroup@483..493
              BindVar@483..487 ":NEW"
              Dot@487..488 "."
              Ident@488..493 "email"
          Comma@493..494 ","
          Whitespace@494..495 " "
          Expression@495..505
            IdentGroup@495..505
              BindVar@495..499 ":NEW"
              Dot@499..500 "."
              Ident@500..505 "phone"
          Comma@505..506 ","
          Whitespace@506..507 " "
          IdentGroup@507..509
            Ident@507..509 "id"
          RParen@509..510 ")"
          Semicolon@510..511 ";"
        Whitespace@511..512 "\n"
      Keyword@512..515 "END"
      Semicolon@515..516 ";"
    Whitespace@516..518 "\n\n"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_schema_trigger() {
        const INPUT: &str = include_str!("../../tests/trigger/schema_trigger.ora.sql");
        check(
            parse(INPUT, parse_trigger),
            expect![[r#"
Root@0..84
  Trigger@0..84
    TriggerHeader@0..63
      Keyword@0..6 "CREATE"
      Whitespace@6..7 " "
      Keyword@7..14 "TRIGGER"
      Whitespace@14..15 " "
      IdentGroup@15..26
        Ident@15..26 "no_drop_trg"
      Whitespace@26..31 "\n    "
      Keyword@31..37 "BEFORE"
      Whitespace@37..38 " "
      Keyword@38..42 "DROP"
      Whitespace@42..43 " "
      Keyword@43..45 "ON"
      Whitespace@45..46 " "
      IdentGroup@46..55
        Ident@46..55 "my_schema"
      Dot@55..56 "."
      Keyword@56..62 "SCHEMA"
      Whitespace@62..63 "\n"
    Block@63..83
      Keyword@63..68 "BEGIN"
      Whitespace@68..73 "\n    "
      BlockStatement@73..78
        Keyword@73..77 "NULL"
        Semicolon@77..78 ";"
      Whitespace@78..79 "\n"
      Keyword@79..82 "END"
      Semicolon@82..83 ";"
    Whitespace@83..84 "\n"
"#]],
            vec![],
        );
    }
}
