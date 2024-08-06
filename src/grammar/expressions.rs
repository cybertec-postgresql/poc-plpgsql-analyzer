// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements different logic/arithmetic SQL expression parser.

//  Heavily inspired by
//    https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
//    https://arzg.github.io/lang/10/

use rowan::Checkpoint;

use crate::grammar::{parse_ident, parse_ident_or_function_invocation};
use crate::parser::{safe_loop, Parser};
use crate::ParseErrorType;
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

/// Attempts to parse an expression if applicable
pub(crate) fn opt_expr(p: &mut Parser) -> bool {
    expr_bp(p, 0).is_ok()
}

/// Parses an expression
pub(crate) fn parse_expr(p: &mut Parser) {
    if let Err(err) = expr_bp(p, 0) {
        p.error(err);
        p.bump_any();
    }
}

/// # Arguments
///
/// * `p`: Mutable reference to the parser instance
/// * `min_bp`: The minimum binding power
///
/// "Binding power" describes the precedence of operators, ranging from weak (low binding power) to strong (high binding power).
///
/// The term `l_bp` (left binding power) refers to the token precedence itself,
/// while `r_bp` (right binding power) describes the subexpression precedence.
///
/// During the main loop, the `l_bp` of the next token is compared to the current `r_bp`.
/// Depending on the result, we may either finish the expression node, or create a sub-expression.
///
/// A binding power of `0` is used to start off the recursion, as well as after encountering parenthesis.
///
fn expr_bp(p: &mut Parser, min_bp: u8) -> Result<(), ParseErrorType> {
    let checkpoint = p.checkpoint();

    let token = p.current();
    match token {
        token
            if (token.is_ident() || token.is_literal())
                // reserved identifiers
                && ![
                    T![and],
                    T![between],
                    T![ilike],
                    T![in],
                    T![like],
                    T![not],
                    T![or],
                    T![then],
                    T![prior],
                    T![connect_by_root]
                ]
                .contains(&token) =>
        {
            match token {
                token if token.is_ident() => {
                    parse_ident_or_function_invocation(p);
                }
                T![bind_var] => {
                    parse_ident(p, 1..2);
                }
                _ => {
                    p.bump_any();
                }
            }
            if min_bp == 0 && (p.at(T![;]) || p.at(T![EOF]) || p.at(T![,])) {
                add_expr_node(p, checkpoint, None);
            }
            p.eat(T![(+)]);
        }
        T!["("] => {
            p.bump_any();
            expr_bp(p, 0)?;
            if !p.expect(T![")"]) {
                p.error(ParseErrorType::UnbalancedParens);
            }
        }
        T![not] | T![+] | T![-] | T![prior] | T![connect_by_root] => {
            if let Some(operator) = prefix_bp(token) {
                match operator.mapping {
                    Some(syntax_kind) => p.bump_any_map(syntax_kind),
                    None => p.bump_any(),
                }
                add_expr_node(p, checkpoint, Some(operator.bp + 1));
            }
        }
        _ => {
            return Err(ParseErrorType::ExpectedOneOfTokens(vec![
                T![unquoted_ident],
                T![quoted_ident],
                T![int_literal],
                T!["("],
                T![-],
                T![not],
                T![+],
                T![quoted_literal],
                T![bind_var],
                T![prior],
                T![connect_by_root],
            ]));
        }
    }

    while !p.at(T![;]) && !p.at(T![EOF]) {
        p.eat(T![not]);
        let op = p.current();

        if let Some(operator) = postfix_bp(op) {
            if operator.bp < min_bp {
                break;
            }

            match operator.mapping {
                Some(syntax_kind) => p.bump_any_map(syntax_kind),
                None => p.bump_any(),
            }

            add_expr_node(p, checkpoint, None);
            continue;
        }

        if let Some(operator) = infix_bp(op) {
            if operator.bp < min_bp {
                break;
            }

            match operator.mapping {
                Some(syntax_kind) => p.bump_any_map(syntax_kind),
                None => p.bump_any(),
            }

            if let Some(cb) = operator.callback {
                cb(p, operator.bp + 1);
            }

            add_expr_node(p, checkpoint, Some(operator.bp + 1));
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

type Callback = &'static dyn Fn(&mut Parser, u8);

struct Operator {
    bp: u8,
    mapping: Option<SyntaxKind>,
    callback: Option<Callback>,
}

impl Operator {
    fn new(bp: u8, mapping: Option<SyntaxKind>, callback: Option<Callback>) -> Self {
        Self {
            bp,
            mapping,
            callback,
        }
    }

    pub fn new_plain(bp: u8) -> Self {
        Self::new(bp, None, None)
    }

    pub fn new_with_map(bp: u8, mapping: SyntaxKind) -> Self {
        Self::new(bp, Some(mapping), None)
    }

    pub fn new_with_cb(bp: u8, callback: Option<Callback>) -> Self {
        Self::new(bp, None, callback)
    }
}

fn prefix_bp(op: TokenKind) -> Option<Operator> {
    Some(match op {
        T![not] => Operator::new_with_map(5, SyntaxKind::LogicOp),
        T![prior] | T![connect_by_root] => Operator::new_with_map(17, SyntaxKind::HierarchicalOp),
        T![+] | T![-] => Operator::new_plain(17),
        _ => return None,
    })
}

fn postfix_bp(op: TokenKind) -> Option<Operator> {
    Some(match op {
        T![!] => Operator::new_plain(19),
        _ => return None,
    })
}

fn infix_bp(op: TokenKind) -> Option<Operator> {
    Some(match op {
        T![or] => Operator::new_with_map(1, SyntaxKind::LogicOp),
        T![and] => Operator::new_with_map(3, SyntaxKind::LogicOp),
        T![=] | T![comparison] => Operator::new_plain(7),
        T![like] | T![ilike] | T![between] | T![in] => Operator::new_with_cb(
            9,
            match op {
                T![between] => Some(&between_cond),
                T![in] => Some(&in_cond),
                _ => None,
            },
        ),
        T![||] => Operator::new_plain(11),
        T![+] | T![-] => Operator::new_plain(13),
        T![*] | T![/] | T![%] => Operator::new_with_map(15, SyntaxKind::ArithmeticOp),
        _ => return None,
    })
}

fn between_cond(p: &mut Parser, min_bp: u8) {
    let _ = expr_bp(p, min_bp);
    p.expect(T![and]);
    let _ = expr_bp(p, min_bp);
}

fn in_cond(p: &mut Parser, min_bp: u8) {
    p.expect(T!["("]);

    safe_loop!(p, {
        let _ = expr_bp(p, min_bp);
        if !p.eat(T![,]) {
            break;
        }
    });

    p.expect(T![")"]);
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::ParseError;
    use crate::ParseErrorType::{ExpectedToken, Incomplete, UnbalancedParens};
    use source_gen::lexer::TokenKind::RParen;

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
            vec![],
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
    IdentGroup@1..2
      Ident@1..2 "a"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_postfix_expr() {
        check(
            parse("a!", parse_expr),
            expect![[r#"
Root@0..2
  Expression@0..2
    IdentGroup@0..1
      Ident@0..1 "a"
    Exclam@1..2 "!"
"#]],
            vec![],
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
      IdentGroup@1..2
        Ident@1..2 "a"
      Exclam@2..3 "!"
"#]],
            vec![],
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
            vec![],
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
      Whitespace@3..4 " "
      Expression@4..10
        Integer@4..5 "1"
        Whitespace@5..6 " "
        ComparisonOp@6..7 ">"
        Whitespace@7..8 " "
        Integer@8..9 "2"
        Whitespace@9..10 " "
    LogicOp@10..13 "AND"
    Whitespace@13..14 " "
    Expression@14..22
      LogicOp@14..17 "NOT"
      Whitespace@17..18 " "
      IdentGroup@18..22
        Ident@18..22 "true"
"#]],
            vec![],
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
    IdentGroup@4..5
      Ident@4..5 "a"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_string_concat() {
        check(
            parse("'1' || a", parse_expr),
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
            vec![],
        );
    }

    #[test]
    fn test_parse_between_expr() {
        check(
            parse("x not between trunc(2) and (1 + 2)", parse_expr),
            expect![[r#"
Root@0..34
  Expression@0..34
    IdentGroup@0..1
      Ident@0..1 "x"
    Whitespace@1..2 " "
    Keyword@2..5 "not"
    Whitespace@5..6 " "
    Keyword@6..13 "between"
    Whitespace@13..14 " "
    FunctionInvocation@14..22
      IdentGroup@14..19
        Ident@14..19 "trunc"
      LParen@19..20 "("
      ArgumentList@20..21
        Argument@20..21
          Integer@20..21 "2"
      RParen@21..22 ")"
    Whitespace@22..23 " "
    Keyword@23..26 "and"
    Whitespace@26..27 " "
    LParen@27..28 "("
    Expression@28..33
      Integer@28..29 "1"
      Whitespace@29..30 " "
      ArithmeticOp@30..31 "+"
      Whitespace@31..32 " "
      Integer@32..33 "2"
    RParen@33..34 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_in_condition() {
        check(
            parse("x not in (1+1, 3)", parse_expr),
            expect![[r#"
Root@0..17
  Expression@0..17
    IdentGroup@0..1
      Ident@0..1 "x"
    Whitespace@1..2 " "
    Keyword@2..5 "not"
    Whitespace@5..6 " "
    Keyword@6..8 "in"
    Whitespace@8..9 " "
    LParen@9..10 "("
    Expression@10..13
      Integer@10..11 "1"
      ArithmeticOp@11..12 "+"
      Integer@12..13 "1"
    Comma@13..14 ","
    Whitespace@14..15 " "
    Integer@15..16 "3"
    RParen@16..17 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_bool_expr() {
        check(
            parse(
                "TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN')",
                parse_expr,
            ),
            expect![[r#"
Root@0..113
  Expression@0..113
    Expression@0..69
      FunctionInvocation@0..28
        IdentGroup@0..7
          Ident@0..7 "TO_CHAR"
        Whitespace@7..8 " "
        LParen@8..9 "("
        ArgumentList@9..27
          Argument@9..16
            Expression@9..16
              IdentGroup@9..16
                Ident@9..16 "SYSDATE"
          Comma@16..17 ","
          Whitespace@17..18 " "
          Argument@18..27
            QuotedLiteral@18..27 "'HH24:MI'"
        RParen@27..28 ")"
      Whitespace@28..29 " "
      Keyword@29..32 "NOT"
      Whitespace@32..33 " "
      Keyword@33..40 "BETWEEN"
      Whitespace@40..41 " "
      QuotedLiteral@41..48 "'08:00'"
      Whitespace@48..49 " "
      Keyword@49..52 "AND"
      Whitespace@52..53 " "
      QuotedLiteral@53..60 "'18:00'"
      Whitespace@60..69 "\n        "
    LogicOp@69..71 "OR"
    Whitespace@71..72 " "
    Expression@72..113
      FunctionInvocation@72..95
        IdentGroup@72..79
          Ident@72..79 "TO_CHAR"
        Whitespace@79..80 " "
        LParen@80..81 "("
        ArgumentList@81..94
          Argument@81..88
            Expression@81..88
              IdentGroup@81..88
                Ident@81..88 "SYSDATE"
          Comma@88..89 ","
          Whitespace@89..90 " "
          Argument@90..94
            QuotedLiteral@90..94 "'DY'"
        RParen@94..95 ")"
      Whitespace@95..96 " "
      Keyword@96..98 "IN"
      Whitespace@98..99 " "
      LParen@99..100 "("
      QuotedLiteral@100..105 "'SAT'"
      Comma@105..106 ","
      Whitespace@106..107 " "
      QuotedLiteral@107..112 "'SUN'"
      RParen@112..113 ")"
"#]],
            vec![],
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
    Whitespace@3..4 " "
    Expression@4..9
      IdentGroup@4..5
        Ident@4..5 "a"
      Whitespace@5..6 " "
      ArithmeticOp@6..7 "*"
      Whitespace@7..8 " "
      Integer@8..9 "2"
"#]],
            vec![],
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
      Whitespace@3..4 " "
      Expression@4..14
        Expression@4..10
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
            vec![],
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
    IdentGroup@5..6
      Ident@5..6 "a"
  RParen@6..7 ")"
"#]],
            vec![],
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
            vec![],
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
            vec![],
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
            vec![],
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
      Whitespace@23..24 " "
      Expression@24..74
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
          Whitespace@38..39 " "
          Expression@39..48
            IdentGroup@39..42
              Ident@39..42 "bar"
            Whitespace@42..43 " "
            ComparisonOp@43..45 ">="
            Whitespace@45..46 " "
            Integer@46..48 "42"
        RParen@48..49 ")"
        Whitespace@49..50 " "
        LogicOp@50..53 "AND"
        Whitespace@53..54 " "
        Expression@54..74
          IdentGroup@54..57
            Ident@54..57 "foo"
          Whitespace@57..58 " "
          ComparisonOp@58..63 "ILIKE"
          Whitespace@63..64 " "
          QuotedLiteral@64..74 "'%stonks%'"
    RParen@74..75 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_qualified_function_invocation() {
        check(
            parse("JOHN.NVL(first_name, 'John')", parse_expr),
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
          Expression@9..19
            IdentGroup@9..19
              Ident@9..19 "first_name"
        Comma@19..20 ","
        Whitespace@20..21 " "
        Argument@21..27
          QuotedLiteral@21..27 "'John'"
      RParen@27..28 ")"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_unbalanced_rparen() {
        check(
            parse("(a < 100))", parse_expr),
            expect![[r#"
Root@0..9
  LParen@0..1 "("
  Expression@1..8
    IdentGroup@1..2
      Ident@1..2 "a"
    Whitespace@2..3 " "
    ComparisonOp@3..4 "<"
    Whitespace@4..5 " "
    Integer@5..8 "100"
  RParen@8..9 ")"
"#]],
            vec![ParseError::new(Incomplete(")".to_string()), 9..10)],
        );
    }

    #[test]
    fn test_parse_unbalanced_lparen() {
        check(
            parse("(a < 100", parse_expr),
            expect![[r#"
Root@0..8
  LParen@0..1 "("
  Expression@1..8
    IdentGroup@1..2
      Ident@1..2 "a"
    Whitespace@2..3 " "
    ComparisonOp@3..4 "<"
    Whitespace@4..5 " "
    Integer@5..8 "100"
"#]],
            vec![
                ParseError::new(ExpectedToken(RParen), 0..0),
                ParseError::new(UnbalancedParens, 0..0),
            ],
        );
    }
}
