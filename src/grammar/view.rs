// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of views from a token tree.

use crate::parser::Parser;
use inner_source_gen::lexer::TokenKind;
use inner_source_gen::syntax::SyntaxKind;

use super::*;

/// Parses a complete view.
pub(crate) fn parse_view(p: &mut Parser) {
    p.start(SyntaxKind::View);

    p.expect(T![create]);
    if p.eat(T![or]) {
        p.expect(T![replace]);
    }

    p.eat(T![no]);
    p.eat(T![force]);

    if p.eat_one_of(&[T![editioning], T![editionable], T![noneditionable]]) {
        p.eat(T![editioning]);
    }

    p.expect(T![view]);

    if p.eat(T![if]) {
        p.expect(T![not]);
        p.expect(T![exists]);
    }

    parse_ident(p, 1..2);

    if p.eat(T![sharing]) {
        p.expect(T![=]);
        p.expect_one_of(&[T![metadata], T![data], T![extended], T![none]]);
        p.eat(T![data]);
    }

    match p.current() {
        T!["("] => {
            p.bump_any();
            safe_loop!(p, {
                if at_out_of_line_constraint(p) {
                    parse_out_of_line_constraint(p);
                } else {
                    parse_ident(p, 1..1);
                    p.eat_one_of(&[T![visible], T![invisible]]);
                    if at_inline_constraint(p) {
                        parse_inline_constraint(p);
                    }
                    if p.at(T![annotations]) {
                        parse_annotations_clause(p);
                    }
                }

                if !p.eat(T![,]) {
                    break;
                }
            });
            p.expect(T![")"]);
        }
        T![of] => match p.nth(1).unwrap_or(T![EOF]) {
            T![xmltype] => parse_xmltype_view_clause(p),
            _ => parse_object_view_clause(p),
        },
        _ => {}
    }

    if p.at(T![annotations]) {
        parse_annotations_clause(p);
    }

    if p.eat(T![default]) {
        p.expect(T![collation]);
        p.expect(T![quoted_literal]);
    }

    if p.eat(T![bequeath]) {
        p.expect_one_of(&[T![current_user], T![definer]]);
    }

    p.expect(T![as]);

    parse_query(p, false);

    if p.eat(T![with]) {
        match p.current() {
            T![check] => {
                p.bump_any();
                p.expect(T![option]);
            }
            T![read] => {
                p.bump_any();
                p.expect(T![only]);
            }
            _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
                T![check],
                T![read],
            ])),
        }

        if p.eat(T![constraint]) {
            parse_ident(p, 1..1);
        }
    }

    p.eat_one_of(&[T![container_map], T![containers_default]]);

    p.eat(T![;]);

    p.finish();
}

fn parse_object_view_clause(p: &mut Parser) {
    p.expect(T![of]);
    parse_ident(p, 1..2);

    match p.current() {
        T![under] => {
            p.bump_any();
            parse_ident(p, 1..2);
        }
        T![with] => {
            p.bump_any();
            p.expect(T![object]);
            p.expect_one_of(&[T![id], T![identifier]]);
            match p.current() {
                T![default] => {
                    p.bump_any();
                }
                T!["("] => {
                    p.bump_any();
                    safe_loop!(p, {
                        parse_ident(p, 1..1);
                        if !p.eat(T![,]) {
                            break;
                        }
                    });
                    p.expect(T![")"]);
                }
                _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
                    T![default],
                    T!["("],
                ])),
            }
        }
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![under],
            T![with],
        ])),
    }

    if p.eat(T!["("]) {
        safe_loop!(p, {
            if at_out_of_line_constraint(p) {
                parse_out_of_line_constraint(p);
            } else {
                parse_ident(p, 1..1);
                if at_inline_constraint(p) {
                    parse_inline_constraint(p);
                }
            }

            if !p.eat(T![,]) {
                break;
            }
        });
        p.expect(T![")"]);
    }
}

fn parse_xmltype_view_clause(p: &mut Parser) {
    p.expect(T![of]);
    p.expect(T![xmltype]);

    if p.eat(T![xmlschema]) {
        p.expect(T![quoted_ident]);
    }

    if p.eat(T![element]) {
        p.expect(T![quoted_ident]);
        if p.eat(T![store]) {
            p.expect(T![all]);
            p.expect(T![varrays]);
            p.expect(T![as]);
            p.expect_one_of(&[T![lobs], T![tables]]);
        }

        if p.eat_one_of(&[T![allow], T![disallow]]) {
            p.expect(T![nonschema]);
        }

        if p.eat_one_of(&[T![allow], T![disallow]]) {
            p.expect(T![anyschema]);
        }
    }

    p.expect(T![with]);
    p.expect(T![object]);
    p.expect_one_of(&[T![id], T![identifier]]);

    match p.current() {
        T!["("] => {
            p.bump_any();
            safe_loop!(p, {
                parse_expr(p);
                if !p.eat(T![,]) {
                    break;
                }
            });
            p.expect(T![")"]);
        }
        T![default] => p.bump_any(),
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T!["("],
            T![default],
        ])),
    }
}

fn parse_annotations_clause(p: &mut Parser) {
    p.expect(T![annotations]);
    p.expect(T!["("]);
    safe_loop!(p, {
        match p.current() {
            T![add] => {
                p.bump_any();
                if p.eat(T![if]) {
                    p.expect(T![not]);
                    p.expect(T![exists]);
                }
            }
            T![drop] => {
                p.bump_any();
                if p.eat(T![if]) {
                    p.expect(T![exists]);
                }
            }
            T![replace] => p.bump_any(),
            _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
                T![add],
                T![drop],
                T![replace],
            ])),
        }

        parse_ident(p, 1..1);
        p.eat(T![quoted_literal]);

        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn parse_simple_view() {
        check(
            parse(
                "CREATE VIEW store_view AS SELECT name FROM stores",
                parse_view,
            ),
            expect![[r#"
Root@0..49
  View@0..49
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "VIEW"
    Whitespace@11..12 " "
    IdentGroup@12..22
      Ident@12..22 "store_view"
    Whitespace@22..23 " "
    Keyword@23..25 "AS"
    Whitespace@25..26 " "
    SelectStmt@26..49
      Keyword@26..32 "SELECT"
      Whitespace@32..33 " "
      SelectClause@33..38
        ColumnExpr@33..38
          IdentGroup@33..37
            Ident@33..37 "name"
          Whitespace@37..38 " "
      Keyword@38..42 "FROM"
      Whitespace@42..43 " "
      IdentGroup@43..49
        Ident@43..49 "stores"
"#]],
            vec![],
        );
    }
}
