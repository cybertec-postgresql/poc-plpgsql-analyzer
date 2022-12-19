// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of procedures from a token tree.

use crate::grammar::{parse_expr, parse_ident};
use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

pub(crate) fn parse_function_invocation(p: &mut Parser) {
    p.start(SyntaxKind::FunctionInvocation);
    parse_ident(p);
    p.expect(TokenKind::LParen);

    if !p.at(TokenKind::RParen) {
        p.start(SyntaxKind::ArgumentList);

        loop {
            match p.current() {
                TokenKind::Comma => {
                    p.bump(TokenKind::Comma);
                }
                TokenKind::RParen | TokenKind::Eof => {
                    break;
                }
                _ => {
                    p.start(SyntaxKind::Argument);
                    parse_expr(p);
                    p.finish();
                }
            }
        }

        p.finish();
    }

    p.expect(TokenKind::RParen);
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn parse_function_call_without_params() {
        check(
            parse("func()", parse_function_invocation),
            expect![[r#"
Root@0..6
  FunctionInvocation@0..6
    Ident@0..4 "func"
    LParen@4..5 "("
    RParen@5..6 ")"
"#]],
        );
    }

    #[test]
    fn parse_function_call_with_one_primitive_arg() {
        check(
            parse("func(123)", parse_function_invocation),
            expect![[r#"
Root@0..9
  FunctionInvocation@0..9
    Ident@0..4 "func"
    LParen@4..5 "("
    ArgumentList@5..8
      Argument@5..8
        Integer@5..8 "123"
    RParen@8..9 ")"
"#]],
        );
    }

    #[test]
    fn parse_function_call_with_sysdate_and_expression() {
        check(
            parse("func(SYSDATE, 1 + 2, 'text')", parse_function_invocation),
            expect![[r#"
Root@0..28
  FunctionInvocation@0..28
    Ident@0..4 "func"
    LParen@4..5 "("
    ArgumentList@5..27
      Argument@5..12
        Ident@5..12 "SYSDATE"
      Comma@12..13 ","
      Whitespace@13..14 " "
      Argument@14..19
        Expression@14..19
          Integer@14..15 "1"
          Whitespace@15..16 " "
          ArithmeticOp@16..17 "+"
          Whitespace@17..18 " "
          Integer@18..19 "2"
      Comma@19..20 ","
      Whitespace@20..21 " "
      Argument@21..27
        QuotedLiteral@21..27 "'text'"
    RParen@27..28 ")"
"#]],
        );
    }

    #[test]
    fn parse_recursive_function_call() {
        check(
            parse("func(1 + func2(123) / 2)", parse_function_invocation),
            expect![[r#"
Root@0..24
  FunctionInvocation@0..24
    Ident@0..4 "func"
    LParen@4..5 "("
    ArgumentList@5..23
      Argument@5..23
        Expression@5..23
          Integer@5..6 "1"
          Whitespace@6..7 " "
          ArithmeticOp@7..8 "+"
          Expression@8..23
            Whitespace@8..9 " "
            FunctionInvocation@9..19
              Ident@9..14 "func2"
              LParen@14..15 "("
              ArgumentList@15..18
                Argument@15..18
                  Integer@15..18 "123"
              RParen@18..19 ")"
            Whitespace@19..20 " "
            ArithmeticOp@20..21 "/"
            Whitespace@21..22 " "
            Integer@22..23 "2"
    RParen@23..24 ")"
"#]],
        );
    }
}
