// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements different logic/arithmetic SQL expression parser.

use crate::lexer::TokenKind;
use crate::parser::{ParseError, Parser};
use crate::syntax::SyntaxKind;

pub(crate) fn parse_expr(p: &mut Parser) {
    p.start(SyntaxKind::Expression);

    let mut nest_level = 0isize;
    while !p.at(TokenKind::SemiColon) && !p.at(TokenKind::Eof) {
        if p.eat(TokenKind::LParen) {
            nest_level += 1;
        }

        if !p.expect_one_of(&[
            TokenKind::Ident,
            TokenKind::QuotedLiteral,
            TokenKind::Integer,
        ]) {
            break;
        }

        p.eat(TokenKind::OracleJoinKw);

        if !p.expect_one_of(&[TokenKind::ComparisonOp, TokenKind::LikeKw]) {
            break;
        }

        if !p.expect_one_of(&[
            TokenKind::Ident,
            TokenKind::QuotedLiteral,
            TokenKind::Integer,
        ]) {
            break;
        }

        if p.eat(TokenKind::RParen) {
            nest_level -= 1;
        }

        p.eat_one_of(&[TokenKind::AndKw, TokenKind::OrKw]);
    }

    if nest_level != 0 {
        p.error(ParseError::UnbalancedParens);
    }

    p.finish();
}

#[cfg(test)]
mod tests {
    use super::super::tests::{check, parse};
    use super::*;
    use expect_test::expect;

    #[test]
    fn test_parse_simple_expr() {
        check(
            parse("a < 100", parse_expr),
            expect![[r#"
Root@0..7
  Expression@0..7
    Ident@0..1 "a"
    Whitespace@1..2 " "
    ComparisonOp@2..3 "<"
    Whitespace@3..4 " "
    Integer@4..7 "100"
"#]],
        );
    }

    #[test]
    fn test_parse_nested_expr() {
        check(
            parse(
                "a < 100 AND (10 <> b OR (c = 'foo' AND bar >= 42) AND foo ILIKE '%stonks%')",
                parse_expr,
            ),
            expect![[r#"
Root@0..75
  Expression@0..75
    Ident@0..1 "a"
    Whitespace@1..2 " "
    ComparisonOp@2..3 "<"
    Whitespace@3..4 " "
    Integer@4..7 "100"
    Whitespace@7..8 " "
    Keyword@8..11 "AND"
    Whitespace@11..12 " "
    LParen@12..13 "("
    Integer@13..15 "10"
    Whitespace@15..16 " "
    ComparisonOp@16..18 "<>"
    Whitespace@18..19 " "
    Ident@19..20 "b"
    Whitespace@20..21 " "
    Keyword@21..23 "OR"
    Whitespace@23..24 " "
    LParen@24..25 "("
    Ident@25..26 "c"
    Whitespace@26..27 " "
    ComparisonOp@27..28 "="
    Whitespace@28..29 " "
    QuotedLiteral@29..34 "'foo'"
    Whitespace@34..35 " "
    Keyword@35..38 "AND"
    Whitespace@38..39 " "
    Ident@39..42 "bar"
    Whitespace@42..43 " "
    ComparisonOp@43..45 ">="
    Whitespace@45..46 " "
    Integer@46..48 "42"
    RParen@48..49 ")"
    Whitespace@49..50 " "
    Keyword@50..53 "AND"
    Whitespace@53..54 " "
    Ident@54..57 "foo"
    Whitespace@57..58 " "
    ComparisonOp@58..63 "ILIKE"
    Whitespace@63..64 " "
    QuotedLiteral@64..74 "'%stonks%'"
    RParen@74..75 ")"
"#]],
        );
    }
}
