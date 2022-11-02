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
    while !p.at(TokenKind::SemiColon) {
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
