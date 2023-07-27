// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of constraints from a token tree.

use super::*;

#[allow(unused)]
/// Parses a complete constraint
pub(crate) fn parse_constraint(p: &mut Parser) {
    p.start(SyntaxKind::Constraint);

    let offset = if p.at(T![constraint]) { 2 } else { 0 };
    match p.nth(offset).unwrap_or(T![EOF]) {
        T![check] | T![not] | T![null] | T![references] => parse_inline_constraint(p),
        T![unique] => match p.nth(offset + 1).unwrap_or(T![EOF]) {
            T!["("] => parse_out_of_line_constraint(p),
            _ => parse_inline_constraint(p),
        },
        T![primary] | T![foreign] => match p.nth(offset + 2).unwrap_or(T![EOF]) {
            T!["("] => parse_out_of_line_constraint(p),
            _ => parse_inline_constraint(p),
        },
        T![scope] => match p.nth(offset + 1).unwrap_or(T![EOF]) {
            T![is] => parse_inline_ref_constraint(p),
            _ => parse_out_of_line_ref_constraint(p),
        },
        T![with] => parse_inline_ref_constraint(p),
        T![ref] => parse_out_of_line_ref_constraint(p),
        t => p.error(ParseErrorType::ExpectedConstraint(t)),
    }

    p.finish();
}

pub(crate) fn at_inline_constraint(p: &mut Parser) -> bool {
    [
        T![check],
        T![constraint],
        T![not],
        T![null],
        T![primary],
        T![references],
        T![unique],
    ]
    .contains(&p.current())
}

pub(crate) fn parse_inline_constraint(p: &mut Parser) {
    if p.eat(T![constraint]) {
        parse_ident(p, 1..1); // TODO: Check 1..1
    }

    match p.current() {
        T![check] => {
            p.bump_any();
            p.expect(T!["("]);
            parse_expr(p);
            p.expect(T![")"]);
        }
        T![not] | T![null] => {
            p.eat(T![not]);
            p.expect(T![null]);
        }
        T![unique] => p.bump_any(),
        T![primary] => {
            p.bump_any();
            p.expect(T![key]);
        }
        T![references] => parse_references_clause(p),
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![check],
            T![not],
            T![null],
            T![primary],
            T![references],
            T![unique],
        ])),
    }

    opt_parse_constraint_state(p);
    p.eat_one_of(&[T![precheck], T![noprecheck]]);
}

pub(crate) fn at_out_of_line_constraint(p: &mut Parser) -> bool {
    [
        T![check],
        T![constraint],
        T![foreign],
        T![primary],
        T![unique],
    ]
    .contains(&p.current())
}

pub(crate) fn parse_out_of_line_constraint(p: &mut Parser) {
    if p.eat(T![constraint]) {
        parse_ident(p, 1..1); // TODO: Check 1..1
    }

    match p.current() {
        T![check] => {
            p.bump_any();
            p.expect(T!["("]);
            parse_expr(p);
            p.expect(T![")"]);
        }
        T![unique] => {
            p.bump_any();
            parse_column_list(p);
        }
        T![primary] => {
            p.bump_any();
            p.expect(T![key]);
            parse_column_list(p);
        }
        T![foreign] => {
            p.bump_any();
            p.expect(T![key]);
            parse_column_list(p);
            parse_references_clause(p);
        }
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![check],
            T![foreign],
            T![primary],
            T![unique],
        ])),
    }

    opt_parse_constraint_state(p);
    p.eat_one_of(&[T![precheck], T![noprecheck]]);
}

fn parse_inline_ref_constraint(p: &mut Parser) {
    match p.current() {
        T![constraint] | T![references] => {
            if p.eat(T![constraint]) {
                parse_ident(p, 1..1); // TODO: Check 1..1
            }
            parse_references_clause(p);
            opt_parse_constraint_state(p);
        }
        T![scope] => {
            p.bump_any();
            p.expect(T![is]);
            parse_ident(p, 1..2);
        }
        T![with] => {
            p.bump_any();
            p.expect(T![rowid]);
        }
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![constraint],
            T![references],
            T![scope],
            T![with],
        ])),
    }
    if p.eat(T![constraint]) {
        parse_ident(p, 1..1); // TODO: Check 1..1
    }
}

fn parse_out_of_line_ref_constraint(p: &mut Parser) {
    match p.current() {
        T![constraint] | T![foreign] => {
            if p.eat(T![constraint]) {
                parse_ident(p, 1..1); // TODO: Check 1..1
            }
            p.expect(T![foreign]);
            p.expect(T![key]);
            parse_column_list(p);

            parse_references_clause(p);
            opt_parse_constraint_state(p);
        }
        T![ref] => {
            p.bump_any();
            p.expect(T!["("]);
            parse_ident(p, 1..1);
            p.expect(T![")"]);
            p.expect(T![with]);
            p.expect(T![rowid]);
        }
        T![scope] => {
            p.bump_any();
            p.expect(T![for]);
            p.expect(T!["("]);
            parse_ident(p, 1..1);
            p.expect(T![")"]);
            p.expect(T![is]);
            parse_ident(p, 1..2);
        }
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![constraint],
            T![foreign],
            T![ref],
            T![scope],
        ])),
    }
    if p.eat(T![constraint]) {
        parse_ident(p, 1..1); // TODO: Check 1..1
    }
}

fn parse_references_clause(p: &mut Parser) {
    p.expect(T![references]);
    parse_ident(p, 1..2);

    if p.at(T!["("]) {
        parse_column_list(p);
    }

    if p.eat(T![on]) {
        p.expect(T![delete]);
        match p.current() {
            T![cascade] => p.bump_any(),
            T![set] => {
                p.bump_any();
                p.expect(T![null]);
            }
            _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
                T![cascade],
                T![set],
            ])),
        }
    }
}

fn opt_parse_constraint_state(p: &mut Parser) {
    match p.current() {
        T![initially] => {
            p.bump_any();
            p.expect_one_of(&[T![deferred], T![immediate]]);
            if p.eat(T![not]) {
                p.expect(T![deferrable]);
            } else {
                p.eat(T![deferrable]);
            }
        }
        T![not] | T![deferrable] => {
            p.eat(T![not]);
            p.expect(T![deferrable]);
            if p.eat(T![initially]) {
                p.expect_one_of(&[T![deferred], T![immediate]]);
            }
        }
        _ => {}
    }

    p.eat_one_of(&[T![rely], T![norely]]);
    if p.at(T![using]) {
        // TODO: using_index_clause
    }
    p.eat_one_of(&[T![enable], T![disable]]);
    p.eat_one_of(&[T![validate], T![novalidate]]);
    if p.eat(T![exceptions]) {
        p.expect(T![into]);
        parse_ident(p, 1..2);
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn parse_unique_constraint() {
        check(
            parse("CONSTRAINT id_u UNIQUE", parse_constraint),
            expect![[r#"
Root@0..22
  Constraint@0..22
    Keyword@0..10 "CONSTRAINT"
    Whitespace@10..11 " "
    IdentGroup@11..15
      Ident@11..15 "id_u"
    Whitespace@15..16 " "
    Keyword@16..22 "UNIQUE"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_primary_constraint() {
        check(
            parse("CONSTRAINT stores PRIMARY KEY (store_id)", parse_constraint),
            expect![[r#"
Root@0..40
  Constraint@0..40
    Keyword@0..10 "CONSTRAINT"
    Whitespace@10..11 " "
    IdentGroup@11..17
      Ident@11..17 "stores"
    Whitespace@17..18 " "
    Keyword@18..25 "PRIMARY"
    Whitespace@25..26 " "
    Keyword@26..29 "KEY"
    Whitespace@29..30 " "
    LParen@30..31 "("
    IdentGroup@31..39
      Ident@31..39 "store_id"
    RParen@39..40 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_not_null_constraint() {
        check(
            parse("CONSTRAINT name NOT NULL", parse_constraint),
            expect![[r#"
Root@0..24
  Constraint@0..24
    Keyword@0..10 "CONSTRAINT"
    Whitespace@10..11 " "
    IdentGroup@11..15
      Ident@11..15 "name"
    Whitespace@15..16 " "
    Keyword@16..19 "NOT"
    Whitespace@19..20 " "
    Keyword@20..24 "NULL"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_references_constraint() {
        check(
            parse(
                "CONSTRAINT fk_storeid REFERENCES stores(store_id)",
                parse_constraint,
            ),
            expect![[r#"
Root@0..49
  Constraint@0..49
    Keyword@0..10 "CONSTRAINT"
    Whitespace@10..11 " "
    IdentGroup@11..21
      Ident@11..21 "fk_storeid"
    Whitespace@21..22 " "
    Keyword@22..32 "REFERENCES"
    Whitespace@32..33 " "
    IdentGroup@33..39
      Ident@33..39 "stores"
    LParen@39..40 "("
    IdentGroup@40..48
      Ident@40..48 "store_id"
    RParen@48..49 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_references_on_delete_constraint() {
        check(
            parse(
                "REFERENCES stores(store_id) ON DELETE CASCADE",
                parse_constraint,
            ),
            expect![[r#"
Root@0..45
  Constraint@0..45
    Keyword@0..10 "REFERENCES"
    Whitespace@10..11 " "
    IdentGroup@11..17
      Ident@11..17 "stores"
    LParen@17..18 "("
    IdentGroup@18..26
      Ident@18..26 "store_id"
    RParen@26..27 ")"
    Whitespace@27..28 " "
    Keyword@28..30 "ON"
    Whitespace@30..31 " "
    Keyword@31..37 "DELETE"
    Whitespace@37..38 " "
    Keyword@38..45 "CASCADE"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_check_constraint() {
        check(
            parse("CHECK (store_id BETWEEN 1 AND 9) DISABLE", parse_constraint),
            expect![[r#"
Root@0..40
  Constraint@0..40
    Keyword@0..5 "CHECK"
    Whitespace@5..6 " "
    LParen@6..7 "("
    Expression@7..31
      IdentGroup@7..15
        Ident@7..15 "store_id"
      Whitespace@15..16 " "
      Keyword@16..23 "BETWEEN"
      Whitespace@23..24 " "
      Integer@24..25 "1"
      Whitespace@25..26 " "
      Keyword@26..29 "AND"
      Whitespace@29..30 " "
      Integer@30..31 "9"
    RParen@31..32 ")"
    Whitespace@32..33 " "
    Keyword@33..40 "DISABLE"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_scope_is_constraint() {
        check(
            parse("SCOPE IS store_obj", parse_constraint),
            expect![[r#"
Root@0..18
  Constraint@0..18
    Keyword@0..5 "SCOPE"
    Whitespace@5..6 " "
    Keyword@6..8 "IS"
    Whitespace@8..9 " "
    IdentGroup@9..18
      Ident@9..18 "store_obj"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_deferrable_constraint() {
        check(
            parse(
                "UNIQUE (store_id) INITIALLY DEFERRED DEFERRABLE",
                parse_constraint,
            ),
            expect![[r#"
Root@0..47
  Constraint@0..47
    Keyword@0..6 "UNIQUE"
    Whitespace@6..7 " "
    LParen@7..8 "("
    IdentGroup@8..16
      Ident@8..16 "store_id"
    RParen@16..17 ")"
    Whitespace@17..18 " "
    Keyword@18..27 "INITIALLY"
    Whitespace@27..28 " "
    Keyword@28..36 "DEFERRED"
    Whitespace@36..37 " "
    Keyword@37..47 "DEFERRABLE"
"#]],
            vec![],
        );
    }
}
