// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>

//! Implements parsing of PL/SQL call specifications from a token tree.
//! Refer to https://docs.oracle.com/en/database/oracle/oracle-database/23/lnpls/call-specification.html#GUID-C5F117AE-E9A2-499B-BA6A-35D072575BAD

use crate::grammar::{parse_expr, parse_ident};
use crate::lexer::{TokenKind, T};
use crate::parser::{safe_loop, Parser};
use crate::ParseErrorType;

/// Attempts to parse a call_spec if applicable
pub(crate) fn opt_call_spec(p: &mut Parser) -> bool {
    if [T![language], T![mle], T![external]].contains(&p.current()) {
        parse_call_spec(p);
        true
    } else {
        false
    }
}

/// Parses a call specification
pub(super) fn parse_call_spec(p: &mut Parser) {
    match p.current() {
        T![language] => match p.nth(1) {
            Some(T![java]) => {
                p.bump_any();
                p.bump_any();
                p.expect(T![name]);
                parse_expr(p);
            }
            Some(T![c]) => parse_c_declaration(p),
            _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![T![c], T![java]])),
        },
        T![mle] => parse_javascript_declaration(p),
        T![external] => parse_c_declaration(p),
        _ => p.error(ParseErrorType::ExpectedOneOfTokens(vec![
            T![language],
            T![mle],
            T![external],
        ])),
    }
}

fn parse_javascript_declaration(p: &mut Parser) {
    p.expect(T![mle]);

    match p.current() {
        T![module] => {
            p.expect(T![module]);
            parse_ident(p, 1..2);

            if p.eat(T![env]) {
                parse_ident(p, 1..2);
            }

            p.expect(T![signature]);
            parse_expr(p);
        }
        T![language] => {
            p.expect(T![language]);
            parse_ident(p, 1..1);
            parse_expr(p);
        }
        _ => {
            p.error(ParseErrorType::ExpectedOneOfTokens(vec![
                T![module],
                T![language],
            ]));
        }
    }
}

fn parse_c_declaration(p: &mut Parser) {
    p.expect_one_of(&[T![external], T![language]]);
    p.eat(T![c]);

    if p.eat(T![name]) {
        parse_ident(p, 1..1);
    }

    p.expect(T![library]);
    parse_ident(p, 1..1);

    if p.eat(T![name]) {
        parse_ident(p, 1..1);
    }

    if p.eat(T![agent]) {
        p.expect(T![in]);
        p.expect(T!["("]);
        safe_loop!(p, {
            parse_ident(p, 1..1);

            if !p.eat(T![,]) {
                break;
            }
        });
        p.expect(T![")"]);
    }

    if p.eat(T![with]) {
        p.expect(T![context]);
    }

    if p.eat(T![parameters]) {
        p.expect(T!["("]);
        safe_loop!(p, {
            match p.current() {
                T![context] => p.bump_any(),
                T![self] => {
                    p.bump_any();
                    if !p.eat(T![tdo]) {
                        parse_opt_c_external_parameter_property(p);
                    }
                }
                _ => {
                    if !p.eat(T![return]) {
                        parse_ident(p, 1..1);
                    }

                    parse_opt_c_external_parameter_property(p);

                    if p.eat(T![by]) {
                        p.expect(T![reference]);
                    }

                    parse_ident(p, 0..1);
                }
            }

            if !p.eat(T![,]) {
                break;
            }
        });
        p.expect(T![")"]);
    }
}

fn parse_opt_c_external_parameter_property(p: &mut Parser) -> bool {
    match p.current() {
        T![indicator] => {
            p.bump_any();
            p.eat_one_of(&[T![struct], T![tdo]]);
            true
        }
        T![length] | T![duration] | T![maxlen] | T![charsetid] | T![charsetform] => {
            p.bump_any();
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_java_call_spec() {
        const INPUT: &str = "LANGUAGE JAVA NAME 'Adjuster.raiseSalary (int, float)'";

        check(
            parse(INPUT, parse_call_spec),
            expect![[r#"
Root@0..54
  Keyword@0..8 "LANGUAGE"
  Whitespace@8..9 " "
  Keyword@9..13 "JAVA"
  Whitespace@13..14 " "
  Keyword@14..18 "NAME"
  Whitespace@18..19 " "
  Expression@19..54
    QuotedLiteral@19..54 "'Adjuster.raiseSalary ..."
"#]],
            vec![],
        );
    }

    #[test]
    fn test_javascript_call_spec() {
        const INPUT: &str = "MLE LANGUAGE JAVASCRIPT 'return 1'";

        check(
            parse(INPUT, parse_call_spec),
            expect![[r#"
Root@0..34
  Keyword@0..3 "MLE"
  Whitespace@3..4 " "
  Keyword@4..12 "LANGUAGE"
  Whitespace@12..13 " "
  IdentGroup@13..23
    Ident@13..23 "JAVASCRIPT"
  Whitespace@23..24 " "
  Expression@24..34
    QuotedLiteral@24..34 "'return 1'"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_javascript_module_call_spec() {
        const INPUT: &str = "MLE MODULE custom_mod SIGNATURE 'mod_func(Out<number>)'";

        check(
            parse(INPUT, parse_call_spec),
            expect![[r#"
Root@0..55
  Keyword@0..3 "MLE"
  Whitespace@3..4 " "
  Keyword@4..10 "MODULE"
  Whitespace@10..11 " "
  IdentGroup@11..21
    Ident@11..21 "custom_mod"
  Whitespace@21..22 " "
  Keyword@22..31 "SIGNATURE"
  Whitespace@31..32 " "
  Expression@32..55
    QuotedLiteral@32..55 "'mod_func(Out<number>)'"
"#]],
            vec![],
        );
    }
}
