// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2022 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements grammar parsing of the token tree from the lexer.

mod function;
mod procedure;
mod query;

pub(crate) use function::*;
pub(crate) use procedure::*;
pub(crate) use query::*;

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;

/// Parses the parameter list in the procedure header
fn parse_param_list(p: &mut Parser) {
    if p.at(TokenKind::LParen) {
        p.start(SyntaxKind::ParamList);
        p.bump(TokenKind::LParen);

        loop {
            match p.current() {
                TokenKind::Comma => {
                    p.bump(TokenKind::Comma);
                }
                TokenKind::RParen | TokenKind::Eof => {
                    break;
                }
                _ => {
                    parse_param(p);
                }
            }
        }

        p.expect(TokenKind::RParen);
        p.finish();
    }
}

/// Parses a single parameter in a parameter list of a procedure.
///
/// Example:
///   p2 VARCHAR2 := 'not empty'
fn parse_param(p: &mut Parser) {
    p.start(SyntaxKind::Param);
    p.expect(TokenKind::Ident);

    while !p.at(TokenKind::RParen) && !p.at(TokenKind::Comma) && !p.at(TokenKind::Eof) {
        p.bump_any();
    }

    p.finish();
}

/// Parses a data type.
fn parse_typename(p: &mut Parser) {
    if p.at(TokenKind::NumberKw) {
        p.eat(TokenKind::NumberKw);
    } else {
        p.expect(TokenKind::Ident);
        p.expect(TokenKind::Percentage);
        p.expect(TokenKind::TypeKw);
    }
}

/// Parses a SQL identifier.
fn parse_ident(p: &mut Parser) {
    p.expect(TokenKind::Ident);
}

#[cfg(test)]
mod tests {
    use crate::parser::{Parse, Parser};
    use expect_test::{expect, Expect};
    use super::*;

    /// Helper function to compare the build syntax tree with the expected output.
    pub fn check(parse: Parse, expected_tree: Expect) {
        expected_tree.assert_eq(parse.tree().as_str())
    }

    /// A helper to allow to call the different parse functions.
    pub fn parse<F>(input: &str, f: F) -> Parse
    where
        F: Fn(&mut Parser),
    {
        let mut parser = Parser::new(input);
        f(&mut parser);
        parser.build()
    }

    #[test]
    fn test_parse_ident() {
        check(
            parse("hello", parse_ident),
            expect![[r#"
Root@0..5
  Ident@0..5 "hello"
"#]],
        );
    }

    #[test]
    fn test_parse_ident_with_trivia() {
        const INPUT: &str = " -- hello\n  foo";
        check(
            parse(INPUT, parse_ident),
            expect![[r#"
Root@0..15
  Whitespace@0..1 " "
  Comment@1..9 "-- hello"
  Whitespace@9..12 "\n  "
  Ident@12..15 "foo"
"#]],
        );
    }

    #[test]
    fn test_parse_param() {
        assert!(parse("p_1 VARCHAR2", parse_param).ok());
        assert!(parse("p_2 NUMBER", parse_param).ok());
        assert!(parse("p_3 IN BOOLEAN := FALSE", parse_param).ok());
        assert!(parse("p_4 IN OUT NOCOPY DATE", parse_param).ok());
        assert!(parse("p_5", parse_param).ok());

        check(
            parse("p_1 VARCHAR2", parse_param),
            expect![[r#"
Root@0..12
  Param@0..12
    Ident@0..3 "p_1"
    Whitespace@3..4 " "
    Ident@4..12 "VARCHAR2"
"#]],
        );

        check(
            parse("  foo bar%type", parse_param),
            expect![[r#"
Root@0..14
  Param@0..14
    Whitespace@0..2 "  "
    Ident@2..5 "foo"
    Whitespace@5..6 " "
    Ident@6..9 "bar"
    Percentage@9..10 "%"
    Keyword@10..14 "type"
"#]],
        );
    }

    #[test]
    fn test_parse_param_with_default_value() {
        check(
            parse("p2 VARCHAR2 := 'not empty'", parse_param),
            expect![[r#"
Root@0..26
  Param@0..26
    Ident@0..2 "p2"
    Whitespace@2..3 " "
    Ident@3..11 "VARCHAR2"
    Whitespace@11..12 " "
    Assign@12..14 ":="
    Whitespace@14..15 " "
    QuotedLiteral@15..26 "'not empty'"
"#]],
        );
    }
}
