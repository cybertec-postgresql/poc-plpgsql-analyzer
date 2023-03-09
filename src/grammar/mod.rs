// SPDX-License-Identifier: SEE LICENSE IN LICENSE.md
// SPDX-FileCopyrightText: 2023 CYBERTEC PostgreSQL International GmbH
// <office@cybertec.at>
// SPDX-FileContributor: Sebastian Ziebell <sebastian.ziebell@asquera.de>

//! Implements grammar parsing of the token tree from the lexer.

use std::ops::Range;

pub(crate) use datatype::*;
pub(crate) use expressions::*;
pub(crate) use function::*;
pub(crate) use function_invocation::*;
pub(crate) use procedure::*;
pub(crate) use query::*;

use crate::lexer::TokenKind;
use crate::parser::Parser;
use crate::syntax::SyntaxKind;
use crate::ParseError;

mod datatype;
mod expressions;
mod function;
mod function_invocation;
mod procedure;
mod query;

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
/// Refer to https://docs.oracle.com/en/database/oracle/oracle-database/21/lnpls/formal-parameter-declaration.html#GUID-5BA8E033-96B9-439A-A4FC-4844FEC14AD8.
///
/// Example:
///   p2 VARCHAR2 := 'not empty'
fn parse_param(p: &mut Parser) {
    p.start(SyntaxKind::Param);
    parse_ident(p, 1..1);

    if !p.at(TokenKind::RParen) && !p.at(TokenKind::Comma) && !p.at(TokenKind::Eof) {
        p.eat(TokenKind::InKw);

        if p.eat(TokenKind::OutKw) {
            p.eat(TokenKind::NoCopyKw);
            parse_datatype(p);
        } else {
            parse_datatype(p);
            if p.eat_one_of(&[TokenKind::Assign, TokenKind::DefaultKw]) {
                parse_expr(p);
            }
        }
    }

    p.finish();
}

/// <https://docs.oracle.com/cd/B28359_01/appdev.111/b28370/block.htm#CJAIABJJ>
fn parse_var_decl_list(p: &mut Parser) {
    if p.at(TokenKind::BeginKw) {
        return;
    }

    p.start(SyntaxKind::VariableDeclList);
    while !p.at(TokenKind::BeginKw) && !p.at(TokenKind::Eof) {
        p.start(SyntaxKind::VariableDecl);

        if !p.expect_one_of(&[TokenKind::UnquotedIdent, TokenKind::QuotedIdent]) {
            break;
        }

        while !p.at(TokenKind::SemiColon) && !p.at(TokenKind::Eof) {
            parse_datatype(p);
        }

        p.finish();

        if !p.eat(TokenKind::SemiColon) {
            break;
        }
    }
    p.finish();
}

/// Parses a qualified SQL identifier.
///
/// # Arguments
///
/// * `p`: The parser struct
/// * `expected_components`: A range of the minimum and maximum expected components that should be present in the identifier.
///     To allow an optional identifier, pass a range starting with `0`.
///
/// returns: ()
///
/// # Examples
///
/// ```
/// // Matches [identifier].<identifier>
/// // parse_qualified_ident(p, 1..2);
/// ```
///
/// ```
/// // Matches <identifier>.<identifier>.<identifier>
/// // parse_qualified_ident(p, 3..3);
/// ```
fn parse_ident(p: &mut Parser, expected_components: Range<u8>) {
    assert!(expected_components.end > 0);
    assert!(expected_components.start <= expected_components.end);

    p.eat_ws();

    if expected_components.start == 0 && !p.current().is_ident() {
        return;
    }

    p.start(SyntaxKind::IdentGroup);

    parse_single_ident(p);

    let mut i: u8 = 1;
    while i < expected_components.end {
        if i < expected_components.start {
            p.expect(TokenKind::Dot);
        } else if p.nth(0) != Some(TokenKind::Dot) {
            break;
        }

        p.expect(TokenKind::Dot);
        parse_single_ident(p);
        i += 1;
    }

    p.finish();
}

/// Helper function for [`parse_ident`]
fn parse_single_ident(p: &mut Parser) {
    if p.current().is_ident() {
        p.bump_any_map(SyntaxKind::Ident)
    } else {
        p.error(ParseError::ExpectedIdent)
    }
}

fn parse_ident_or_function_invocation(p: &mut Parser) {
    if p.nth(1) == Some(TokenKind::LParen) {
        parse_function_invocation(p);
    } else {
        parse_ident(p, 1..3);
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use crate::parser::{Parse, Parser};

    use super::*;

    /// A helper to allow to call the different parse functions.
    pub fn parse<F>(input: &str, f: F) -> Parse
    where
        F: Fn(&mut Parser),
    {
        let mut parser = Parser::new(input);
        f(&mut parser);
        parser.build()
    }

    /// Helper function to compare the build syntax tree with the expected
    /// output.
    pub fn check(parse: Parse, expected_tree: Expect) {
        expected_tree.assert_eq(&format!("{:#?}", parse.syntax()))
    }

    #[test]
    fn test_parse_ident() {
        check(
            parse("hello", |p| parse_ident(p, 1..1)),
            expect![[r#"
Root@0..5
  IdentGroup@0..5
    Ident@0..5 "hello"
"#]],
        );
    }

    #[test]
    fn test_parse_keyword_as_ident() {
        check(
            parse("procedure", |p| parse_ident(p, 1..1)),
            expect![[r#"
Root@0..9
  IdentGroup@0..9
    Ident@0..9 "procedure"
"#]],
        );
    }

    #[test]
    fn test_parse_ident_with_trivia() {
        check(
            parse(" -- hello\n  foo", |p| parse_ident(p, 1..1)),
            expect![[r#"
Root@0..15
  Whitespace@0..1 " "
  Comment@1..9 "-- hello"
  Whitespace@9..12 "\n  "
  IdentGroup@12..15
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
    IdentGroup@0..3
      Ident@0..3 "p_1"
    Whitespace@3..4 " "
    Datatype@4..12
      Keyword@4..12 "VARCHAR2"
"#]],
        );

        check(
            parse("  foo bar%type", parse_param),
            expect![[r#"
Root@0..14
  Param@0..14
    Whitespace@0..2 "  "
    IdentGroup@2..5
      Ident@2..5 "foo"
    Whitespace@5..6 " "
    Datatype@6..14
      IdentGroup@6..9
        Ident@6..9 "bar"
      TypeAttribute@9..14
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
    IdentGroup@0..2
      Ident@0..2 "p2"
    Whitespace@2..3 " "
    Datatype@3..12
      Keyword@3..11 "VARCHAR2"
      Whitespace@11..12 " "
    Assign@12..14 ":="
    Expression@14..26
      Whitespace@14..15 " "
      QuotedLiteral@15..26 "'not empty'"
"#]],
        );
    }

    #[test]
    fn test_parse_variable_declaration_list() {
        const INPUT: &str = "
    l_total_sales NUMBER(15,2);
    l_credit_limit NUMBER (10,0);
    l_contact_name VARCHAR2(255);
";

        check(
            parse(INPUT, parse_var_decl_list),
            expect![[r#"
Root@0..101
  Whitespace@0..5 "\n    "
  VariableDeclList@5..101
    VariableDecl@5..31
      Ident@5..18 "l_total_sales"
      Whitespace@18..19 " "
      Datatype@19..31
        Keyword@19..25 "NUMBER"
        LParen@25..26 "("
        Integer@26..28 "15"
        Comma@28..29 ","
        Integer@29..30 "2"
        RParen@30..31 ")"
    SemiColon@31..32 ";"
    Whitespace@32..37 "\n    "
    VariableDecl@37..65
      Ident@37..51 "l_credit_limit"
      Whitespace@51..52 " "
      Datatype@52..65
        Keyword@52..58 "NUMBER"
        Whitespace@58..59 " "
        LParen@59..60 "("
        Integer@60..62 "10"
        Comma@62..63 ","
        Integer@63..64 "0"
        RParen@64..65 ")"
    SemiColon@65..66 ";"
    Whitespace@66..71 "\n    "
    VariableDecl@71..99
      Ident@71..85 "l_contact_name"
      Whitespace@85..86 " "
      Datatype@86..99
        Keyword@86..94 "VARCHAR2"
        LParen@94..95 "("
        Integer@95..98 "255"
        RParen@98..99 ")"
    SemiColon@99..100 ";"
    Whitespace@100..101 "\n"
"#]],
        );
    }
}
