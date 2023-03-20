// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements different logic/arithmetic SQL expression parser.

//  Heavily inspired by
//    https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
//    https://arzg.github.io/lang/10/

use rowan::Checkpoint;

use crate::grammar::parse_ident_or_function_invocation;
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseError;

/// Attempts to parse an expression if applicable
pub(crate) fn opt_expr(p: &mut Parser) -> bool {
    expr_bp(p, 0).is_ok()
}

/// Parses an expression
pub(crate) fn expr(p: &mut Parser) {
    if let Err(err) = expr_bp(p, 0) {
        p.error(err);
    }
}

fn expr_bp(p: &mut Parser, min_bp: u8) -> Result<(), ParseError> {
    let checkpoint = p.checkpoint();

    let token = p.current();
    match token {
        TokenKind::UnquotedIdent
        | TokenKind::QuotedIdent
        | TokenKind::QuotedLiteral
        | TokenKind::Integer => {
            match token {
                TokenKind::UnquotedIdent | TokenKind::QuotedIdent => {
                    parse_ident_or_function_invocation(p);
                }
                _ => {
                    p.bump_any();
                }
            }
            if min_bp == 0 && (p.at(TokenKind::SemiColon) || p.at(TokenKind::Eof)) {
                add_expr_node(p, checkpoint, None);
            }
            p.eat(TokenKind::OracleJoinKw);
        }
        TokenKind::LParen => {
            p.bump_any();
            let _ = expr_bp(p, 0);
            if !p.expect(TokenKind::RParen) {
                p.error(ParseError::UnbalancedParens);
            }
        }
        TokenKind::NotKw | TokenKind::Plus | TokenKind::Minus => {
            if let Some((r_bp, syntax_kind)) = prefix_bp(token) {
                match syntax_kind {
                    Some(syntax_kind) => p.bump_any_map(syntax_kind),
                    None => p.bump_any(),
                }
                add_expr_node(p, checkpoint, Some(r_bp));
            }
        }
        _ => {
            return Err(ParseError::ExpectedOneOfTokens(vec![
                TokenKind::UnquotedIdent,
                TokenKind::QuotedIdent,
                TokenKind::Integer,
                TokenKind::LParen,
                TokenKind::Minus,
                TokenKind::NotKw,
                TokenKind::Plus,
                TokenKind::QuotedLiteral,
            ]));
        }
    }

    while !p.at(TokenKind::SemiColon) && !p.at(TokenKind::Eof) {
        let op = p.current();

        if let Some((l_bp, syntax_kind)) = postfix_bp(op) {
            if l_bp < min_bp {
                break;
            }

            match syntax_kind {
                Some(syntax_kind) => p.bump_any_map(syntax_kind),
                None => p.bump_any(),
            }

            add_expr_node(p, checkpoint, None);
            continue;
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
    Ok(())
}

fn add_expr_node(p: &mut Parser, checkpoint: Checkpoint, sub_expr: Option<u8>) {
    p.start_node_at(checkpoint, SyntaxKind::Expression);

    if let Some(min_bp) = sub_expr {
        let _ = expr_bp(p, min_bp);
    };

    p.finish();
}

fn prefix_bp(op: TokenKind) -> Option<(u8, Option<SyntaxKind>)> {
    match op {
        TokenKind::NotKw => Some((5, Some(SyntaxKind::LogicOp))),
        TokenKind::Plus | TokenKind::Minus => Some((17, None)),
        _ => None,
    }
}

fn postfix_bp(op: TokenKind) -> Option<(u8, Option<SyntaxKind>)> {
    match op {
        TokenKind::Exclam => Some((19, None)),
        _ => None,
    }
}

fn infix_bp(op: TokenKind) -> Option<((u8, u8), Option<SyntaxKind>)> {
    match op {
        TokenKind::OrKw => Some(((1, 2), Some(SyntaxKind::LogicOp))),
        TokenKind::AndKw => Some(((3, 4), Some(SyntaxKind::LogicOp))),
        TokenKind::ComparisonOp => Some(((7, 8), None)),
        TokenKind::LikeKw => Some(((9, 10), None)),
        TokenKind::DoublePipe => Some(((11, 12), None)),
        TokenKind::Plus | TokenKind::Minus => Some(((13, 14), None)),
        TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percentage => {
            Some(((15, 16), Some(SyntaxKind::ArithmeticOp)))
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
            parse("1", expr),
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
            parse("-a", expr),
            expect![[r#"
Root@0..2
  Expression@0..2
    ArithmeticOp@0..1 "-"
    IdentGroup@1..2
      Ident@1..2 "a"
"#]],
        );
    }

    #[test]
    fn test_parse_postfix_expr() {
        check(
            parse("a!", expr),
            expect![[r#"
Root@0..2
  Expression@0..2
    IdentGroup@0..1
      Ident@0..1 "a"
    Exclam@1..2 "!"
"#]],
        );
    }

    #[test]
    fn test_parse_pre_and_postfix_expr() {
        check(
            parse("-a!", expr),
            expect![[r#"
Root@0..3
  Expression@0..3
    ArithmeticOp@0..1 "-"
    Expression@1..3
      IdentGroup@1..2
        Ident@1..2 "a"
      Exclam@2..3 "!"
"#]],
        );
    }

    #[test]
    fn test_unary_op_in_paren() {
        check(
            parse("((-a)!) + 2", expr),
            expect![[r#"
Root@0..11
  Expression@0..11
    LParen@0..1 "("
    Expression@1..6
      LParen@1..2 "("
      Expression@2..4
        ArithmeticOp@2..3 "-"
        IdentGroup@3..4
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
            parse("NOT 1 > 2 AND NOT true", expr),
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
      IdentGroup@18..22
        Ident@18..22 "true"
"#]],
        );
    }

    #[test]
    fn test_parse_simple_expr() {
        check(
            parse("1 + a", expr),
            expect![[r#"
Root@0..5
  Expression@0..5
    Integer@0..1 "1"
    Whitespace@1..2 " "
    ArithmeticOp@2..3 "+"
    Whitespace@3..4 " "
    IdentGroup@4..5
      Ident@4..5 "a"
"#]],
        );
    }

    #[test]
    fn test_parse_string_concat() {
        check(
            parse("'1' || a", expr),
            expect![[r#"
Root@0..8
  Expression@0..8
    QuotedLiteral@0..3 "'1'"
    Whitespace@3..4 " "
    Concat@4..6 "||"
    Whitespace@6..7 " "
    IdentGroup@7..8
      Ident@7..8 "a"
"#]],
        );
    }

    #[test]
    fn test_parse_op_precedence() {
        check(
            parse("1 + a * 2", expr),
            expect![[r#"
Root@0..9
  Expression@0..9
    Integer@0..1 "1"
    Whitespace@1..2 " "
    ArithmeticOp@2..3 "+"
    Expression@3..9
      Whitespace@3..4 " "
      IdentGroup@4..5
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
            parse("1 + 2 * 3 / 4 - 5", expr),
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
            parse("(1 + a)", expr),
            expect![[r#"
Root@0..7
  LParen@0..1 "("
  Expression@1..6
    Integer@1..2 "1"
    Whitespace@2..3 " "
    ArithmeticOp@3..4 "+"
    Whitespace@4..5 " "
    IdentGroup@5..6
      Ident@5..6 "a"
  RParen@6..7 ")"
"#]],
        );
    }

    #[test]
    fn test_redundant_parens() {
        check(
            parse("(((1)))", expr),
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
            parse("a * (1 + 2) / b", expr),
            expect![[r#"
Root@0..15
  Expression@0..15
    Expression@0..12
      IdentGroup@0..1
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
    IdentGroup@14..15
      Ident@14..15 "b"
"#]],
        );
    }

    #[test]
    fn test_nested_paren() {
        check(
            parse("1 * (2 + (3 + 4))", expr),
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
                expr,
            ),
            expect![[r#"
Root@0..75
  Expression@0..75
    Expression@0..8
      IdentGroup@0..1
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
        IdentGroup@19..20
          Ident@19..20 "b"
        Whitespace@20..21 " "
      LogicOp@21..23 "OR"
      Expression@23..74
        Whitespace@23..24 " "
        LParen@24..25 "("
        Expression@25..48
          Expression@25..35
            IdentGroup@25..26
              Ident@25..26 "c"
            Whitespace@26..27 " "
            ComparisonOp@27..28 "="
            Whitespace@28..29 " "
            QuotedLiteral@29..34 "'foo'"
            Whitespace@34..35 " "
          LogicOp@35..38 "AND"
          Expression@38..48
            Whitespace@38..39 " "
            IdentGroup@39..42
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
          IdentGroup@54..57
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
    fn test_parse_qualified_function_invocation() {
        check(
            parse("JOHN.NVL(first_name, 'John')", expr),
            expect![[r#"
Root@0..28
  Expression@0..28
    FunctionInvocation@0..28
      IdentGroup@0..8
        Ident@0..4 "JOHN"
        Dot@4..5 "."
        Ident@5..8 "NVL"
      LParen@8..9 "("
      ArgumentList@9..27
        Argument@9..19
          IdentGroup@9..19
            Ident@9..19 "first_name"
        Comma@19..20 ","
        Whitespace@20..21 " "
        Argument@21..27
          QuotedLiteral@21..27 "'John'"
      RParen@27..28 ")"
"#]],
        );
    }

    #[test]
    fn test_parse_unbalanced_rparen() {
        check(
            parse("(a < 100))", expr),
            expect![[r#"
Root@0..38
  LParen@0..1 "("
  Expression@1..8
    IdentGroup@1..2
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
            parse("(a < 100", expr),
            expect![[r#"
Root@0..67
  LParen@0..1 "("
  Expression@1..8
    IdentGroup@1..2
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
