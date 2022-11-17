// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements different logic/arithmetic SQL expression parser.

//  Heavily inspired by
//    https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
//    https://arzg.github.io/lang/10/

use rowan::Checkpoint;

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseError;

pub(crate) fn parse_expr(p: &mut Parser) {
    expr_bp(p, 0);
}

fn expr_bp(p: &mut Parser, min_bp: u8) {
    let checkpoint = p.checkpoint();

    let token = p.current();
    match token {
        TokenKind::Ident | TokenKind::QuotedLiteral | TokenKind::Integer => {
            p.bump_any();
            if min_bp == 0 && (p.at(TokenKind::SemiColon) || p.at(TokenKind::Eof)) {
                add_expr_node(p, checkpoint, None);
            }
            p.eat(TokenKind::OracleJoinKw);
        }
        TokenKind::LParen => {
            p.bump_any();
            expr_bp(p, 0);
            if !p.expect(TokenKind::RParen) {
                p.error(ParseError::UnbalancedParens);
            }
        }
        TokenKind::NotKw | TokenKind::Plus | TokenKind::Minus => {
            let (((), r_bp), syntax_kind) = prefix_bp(token);
            match syntax_kind {
                Some(syntax_kind) => p.bump_any_map(syntax_kind),
                None => p.bump_any(),
            }
            add_expr_node(p, checkpoint, Some(r_bp));
        }
        t => {
            p.error(ParseError::ExpectedOneOfTokens(
                [
                    TokenKind::Ident,
                    TokenKind::Integer,
                    TokenKind::LParen,
                    TokenKind::Minus,
                    TokenKind::NotKw,
                    TokenKind::Plus,
                    TokenKind::QuotedLiteral,
                ]
                .to_vec(),
            ));
            return;
        }
    }

    while !p.at(TokenKind::SemiColon) && !p.at(TokenKind::Eof) {
        let op = p.current();

        if let Some(((l_bp, ()), syntax_kind)) = postfix_bp(op) {
            if l_bp < min_bp {
                break;
            }

            match syntax_kind {
                Some(syntax_kind) => p.bump_any_map(syntax_kind),
                None => p.bump_any(),
            }

            add_expr_node(p, checkpoint, None);
            return;
        }

        if let Some(((l_bp, r_bp), syntax_kind)) = infix_bp(op) {
            if l_bp < min_bp {
                break;
            }

            match syntax_kind {
                Some(syntax_kind) => p.bump_any_map(syntax_kind),
                None => p.bump_any(),
            }

            add_expr_node(p, checkpoint, Some(r_bp));
            continue;
        }

        break;
    }
}

fn add_expr_node(p: &mut Parser, checkpoint: Checkpoint, sub_expr: Option<u8>) {
    p.start_node_at(checkpoint, SyntaxKind::Expression);

    if let Some(min_bp) = sub_expr {
        expr_bp(p, min_bp)
    };

    p.finish();
}

fn prefix_bp(op: TokenKind) -> (((), u8), Option<SyntaxKind>) {
    match op {
        TokenKind::NotKw => (((), 5), Some(SyntaxKind::LogicOp)),
        TokenKind::Plus | TokenKind::Minus => (((), 15), None),
        _ => panic!("bad op: {:?}", op),
    }
}

fn postfix_bp(op: TokenKind) -> Option<((u8, ()), Option<SyntaxKind>)> {
    match op {
        TokenKind::Exclam => Some(((17, ()), None)),
        _ => None,
    }
}

fn infix_bp(op: TokenKind) -> Option<((u8, u8), Option<SyntaxKind>)> {
    match op {
        TokenKind::OrKw => Some(((1, 2), Some(SyntaxKind::LogicOp))),
        TokenKind::AndKw => Some(((3, 4), Some(SyntaxKind::LogicOp))),
        TokenKind::ComparisonOp => Some(((7, 8), None)),
        TokenKind::LikeKw => Some(((9, 10), None)),
        TokenKind::Plus | TokenKind::Minus => Some(((11, 12), None)),
        TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percentage => {
            Some(((13, 14), Some(SyntaxKind::ArithmeticOp)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_parse_literal() {
        check(
            parse("1", parse_expr),
            expect![[r#"
Root@0..1
  Expression@0..1
    Integer@0..1 "1"
"#]],
        );
    }

    #[test]
    fn test_parse_prefix_expr() {
        check(
            parse("-a", parse_expr),
            expect![[r#"
Root@0..2
  Expression@0..2
    ArithmeticOp@0..1 "-"
    Ident@1..2 "a"
"#]],
        );
    }

    #[test]
    fn test_parse_postfix_expr() {
        check(
            parse("a!", parse_expr),
            expect![[r#"
Root@0..2
  Expression@0..2
    Ident@0..1 "a"
    Exclam@1..2 "!"
"#]],
        );
    }

    #[test]
    fn test_parse_pre_and_postfix_expr() {
        check(
            parse("-a!", parse_expr),
            expect![[r#"
Root@0..3
  Expression@0..3
    ArithmeticOp@0..1 "-"
    Expression@1..3
      Ident@1..2 "a"
      Exclam@2..3 "!"
"#]],
        );
    }

    #[test]
    fn test_unary_op_in_paren() {
        check(
            parse("((-a)!) + 2", parse_expr),
            expect![[r#"
Root@0..11
  Expression@0..11
    LParen@0..1 "("
    Expression@1..6
      LParen@1..2 "("
      Expression@2..4
        ArithmeticOp@2..3 "-"
        Ident@3..4 "a"
      RParen@4..5 ")"
      Exclam@5..6 "!"
    RParen@6..7 ")"
    Whitespace@7..8 " "
    ArithmeticOp@8..9 "+"
    Whitespace@9..10 " "
    Integer@10..11 "2"
"#]],
        );
    }

    #[test]
    fn test_not_precedence() {
        check(
            parse("NOT 1 > 2 AND NOT true", parse_expr),
            expect![[r#"
Root@0..22
  Expression@0..22
    Expression@0..10
      LogicOp@0..3 "NOT"
      Expression@3..10
        Whitespace@3..4 " "
        Integer@4..5 "1"
        Whitespace@5..6 " "
        ComparisonOp@6..7 ">"
        Whitespace@7..8 " "
        Integer@8..9 "2"
        Whitespace@9..10 " "
    LogicOp@10..13 "AND"
    Expression@13..22
      Whitespace@13..14 " "
      LogicOp@14..17 "NOT"
      Whitespace@17..18 " "
      Ident@18..22 "true"
"#]],
        );
    }

    #[test]
    fn test_parse_simple_expr() {
        check(
            parse("1 + a", parse_expr),
            expect![[r#"
Root@0..5
  Expression@0..5
    Integer@0..1 "1"
    Whitespace@1..2 " "
    ArithmeticOp@2..3 "+"
    Whitespace@3..4 " "
    Ident@4..5 "a"
"#]],
        );
    }

    #[test]
    fn test_parse_op_precedence() {
        check(
            parse("1 + a * 2", parse_expr),
            expect![[r#"
Root@0..9
  Expression@0..9
    Integer@0..1 "1"
    Whitespace@1..2 " "
    ArithmeticOp@2..3 "+"
    Expression@3..9
      Whitespace@3..4 " "
      Ident@4..5 "a"
      Whitespace@5..6 " "
      ArithmeticOp@6..7 "*"
      Whitespace@7..8 " "
      Integer@8..9 "2"
"#]],
        );
    }

    #[test]
    fn test_parse_mirrored_op_precedence() {
        check(
            parse("1 + 2 * 3 / 4 - 5", parse_expr),
            expect![[r#"
Root@0..17
  Expression@0..17
    Expression@0..14
      Integer@0..1 "1"
      Whitespace@1..2 " "
      ArithmeticOp@2..3 "+"
      Expression@3..14
        Expression@3..10
          Whitespace@3..4 " "
          Integer@4..5 "2"
          Whitespace@5..6 " "
          ArithmeticOp@6..7 "*"
          Whitespace@7..8 " "
          Integer@8..9 "3"
          Whitespace@9..10 " "
        ArithmeticOp@10..11 "/"
        Whitespace@11..12 " "
        Integer@12..13 "4"
        Whitespace@13..14 " "
    ArithmeticOp@14..15 "-"
    Whitespace@15..16 " "
    Integer@16..17 "5"
"#]],
        );
    }

    #[test]
    fn test_parse_simple_paren_expr() {
        check(
            parse("(1 + a)", parse_expr),
            expect![[r#"
Root@0..7
  LParen@0..1 "("
  Expression@1..6
    Integer@1..2 "1"
    Whitespace@2..3 " "
    ArithmeticOp@3..4 "+"
    Whitespace@4..5 " "
    Ident@5..6 "a"
  RParen@6..7 ")"
"#]],
        );
    }

    #[test]
    fn test_redundant_parens() {
        check(
            parse("(((1)))", parse_expr),
            expect![[r#"
Root@0..7
  LParen@0..1 "("
  LParen@1..2 "("
  LParen@2..3 "("
  Integer@3..4 "1"
  RParen@4..5 ")"
  RParen@5..6 ")"
  RParen@6..7 ")"
"#]],
        );
    }

    #[test]
    fn test_paren_precedence() {
        check(
            parse("a * (1 + 2) / b", parse_expr),
            expect![[r#"
Root@0..15
  Expression@0..15
    Expression@0..12
      Ident@0..1 "a"
      Whitespace@1..2 " "
      ArithmeticOp@2..3 "*"
      Whitespace@3..4 " "
      LParen@4..5 "("
      Expression@5..10
        Integer@5..6 "1"
        Whitespace@6..7 " "
        ArithmeticOp@7..8 "+"
        Whitespace@8..9 " "
        Integer@9..10 "2"
      RParen@10..11 ")"
      Whitespace@11..12 " "
    ArithmeticOp@12..13 "/"
    Whitespace@13..14 " "
    Ident@14..15 "b"
"#]],
        );
    }

    #[test]
    fn test_nested_paren() {
        check(
            parse("1 * (2 + (3 + 4))", parse_expr),
            expect![[r#"
Root@0..17
  Expression@0..17
    Integer@0..1 "1"
    Whitespace@1..2 " "
    ArithmeticOp@2..3 "*"
    Whitespace@3..4 " "
    LParen@4..5 "("
    Expression@5..16
      Integer@5..6 "2"
      Whitespace@6..7 " "
      ArithmeticOp@7..8 "+"
      Whitespace@8..9 " "
      LParen@9..10 "("
      Expression@10..15
        Integer@10..11 "3"
        Whitespace@11..12 " "
        ArithmeticOp@12..13 "+"
        Whitespace@13..14 " "
        Integer@14..15 "4"
      RParen@15..16 ")"
    RParen@16..17 ")"
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
    Expression@0..8
      Ident@0..1 "a"
      Whitespace@1..2 " "
      ComparisonOp@2..3 "<"
      Whitespace@3..4 " "
      Integer@4..7 "100"
      Whitespace@7..8 " "
    LogicOp@8..11 "AND"
    Whitespace@11..12 " "
    LParen@12..13 "("
    Expression@13..74
      Expression@13..21
        Integer@13..15 "10"
        Whitespace@15..16 " "
        ComparisonOp@16..18 "<>"
        Whitespace@18..19 " "
        Ident@19..20 "b"
        Whitespace@20..21 " "
      LogicOp@21..23 "OR"
      Expression@23..74
        Whitespace@23..24 " "
        LParen@24..25 "("
        Expression@25..48
          Expression@25..35
            Ident@25..26 "c"
            Whitespace@26..27 " "
            ComparisonOp@27..28 "="
            Whitespace@28..29 " "
            QuotedLiteral@29..34 "'foo'"
            Whitespace@34..35 " "
          LogicOp@35..38 "AND"
          Expression@38..48
            Whitespace@38..39 " "
            Ident@39..42 "bar"
            Whitespace@42..43 " "
            ComparisonOp@43..45 ">="
            Whitespace@45..46 " "
            Integer@46..48 "42"
        RParen@48..49 ")"
        Whitespace@49..50 " "
        LogicOp@50..53 "AND"
        Expression@53..74
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

    #[test]
    fn test_parse_unbalanced_rparen() {
        check(
            parse("(a < 100))", parse_expr),
            expect![[r#"
Root@0..38
  LParen@0..1 "("
  Expression@1..8
    Ident@1..2 "a"
    Whitespace@2..3 " "
    ComparisonOp@3..4 "<"
    Whitespace@4..5 " "
    Integer@5..8 "100"
  RParen@8..9 ")"
  Error@9..38
    Text@9..38 "Incomplete input; unp ..."
"#]],
        );
    }

    #[test]
    fn test_parse_unbalanced_lparen() {
        check(
            parse("(a < 100", parse_expr),
            expect![[r#"
Root@0..67
  LParen@0..1 "("
  Expression@1..8
    Ident@1..2 "a"
    Whitespace@2..3 " "
    ComparisonOp@3..4 "<"
    Whitespace@4..5 " "
    Integer@5..8 "100"
  Error@8..31
    Text@8..31 "Expected token 'RParen'"
  Error@31..67
    Text@31..67 "Unbalanced pair of pa ..."
"#]],
        );
    }
}
